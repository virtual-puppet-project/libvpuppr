/*!
A receiver for [MeowFace](https://play.google.com/store/apps/details?id=com.suvidriel.meowface) data.
*/

use godot::{engine::global::Error, prelude::*};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket},
    sync::mpsc::{self, Receiver, Sender},
    thread::JoinHandle,
    time::Duration,
};

use crate::{
    gstring,
    puppets::{puppet_2d::Puppet2d, puppet_3d::Puppet3d, Visitor},
    vstring, Logger,
};

use super::Receiver as GodotReceiver;

// static SEND_DATA: Lazy<Vec<u8>> = Lazy::new(|| {
//     serde_json::to_string(&serde_json::json!({
//         "messageType": "iOSTrackingDataRequest",
//         "time": 1.0,
//         "sentBy": "vpuppr",
//         "ports": [21412]
//     }))
//     .unwrap()
//     .as_bytes()
//     .to_vec()
// });

// TODO maybe increment time?
static SEND_DATA: &str = "{
    \"messageType\": \"iOSTrackingDataRequest\",
    \"time\": 1.0,
    \"sentBy\": \"vpuppr\",
    \"ports\": [21412]
}";

#[derive(Debug, Serialize, Deserialize)]
struct InData {
    timestamp: u32,
    hotkey: i32,
    face_found: bool,
    rotation: Vector3,
    position: Vector3,
    eye_left: Vector3,
    eye_right: Vector3,
    blend_shapes: Vec<InBlendShape>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InBlendShape {
    k: String,
    v: f32,
}

#[derive(Debug, Default)]
pub(crate) struct Data {
    pub blend_shapes: HashMap<String, f32>,

    pub head_rotation: Vector3,
    pub head_position: Vector3,

    pub left_eye_rotation: Vector3,
    pub right_eye_rotation: Vector3,
}

impl From<InData> for Data {
    fn from(value: InData) -> Self {
        Self {
            blend_shapes: HashMap::from_iter(value.blend_shapes.into_iter().map(|v| (v.k, v.v))),

            head_rotation: value.rotation,
            head_position: value.position,

            left_eye_rotation: value.eye_left,
            right_eye_rotation: value.eye_right,
        }
    }
}

#[derive(Debug, GodotClass)]
pub(crate) struct MeowFace {
    pub(crate) data: Data,
    logger: Gd<Logger>,

    ip_address: Option<SocketAddrV4>,
    receive_handle: Option<JoinHandle<()>>,
    thread_killer: Option<Sender<()>>,
    receiver: Option<Receiver<Data>>,
}

#[godot_api]
impl RefCountedVirtual for MeowFace {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::new()
    }
}

impl GodotReceiver<MeowFace> for MeowFace {
    fn create(data: &Dictionary) -> Option<Gd<MeowFace>> {
        let mut meow_face = Self::new();

        let logger = meow_face.logger.bind();

        let address = match data.get("address") {
            Some(v) => {
                if v.get_type() == VariantType::String {
                    v.stringify()
                } else {
                    logger.error("Unable to convert address to string.");
                    return None;
                }
            }
            None => {
                logger.error("MeowFace expected an 'address'.");
                return None;
            }
        };
        let port = match data.get("port") {
            Some(v) => {
                if v.get_type() == VariantType::String {
                    v.stringify()
                } else {
                    logger.error("Unable to convert port to string.");
                    return None;
                }
            }
            None => {
                logger.error("MeowFace expected a 'port'.");
                return None;
            }
        };

        let ip_address = match format!("{}:{}", address, port).parse::<SocketAddrV4>() {
            Ok(v) => v,
            Err(e) => {
                logger.error(format!("{e}"));
                return None;
            }
        };

        meow_face.ip_address = Some(ip_address);

        drop(logger);

        Some(Gd::new(meow_face))
    }

    fn start(&mut self) -> Error {
        let logger = self.logger.bind();

        logger.info("Starting MeowFace!");

        if self.ip_address.is_none() {
            return Error::ERR_UNCONFIGURED;
        }

        let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 21412)) {
            Ok(v) => v,
            Err(e) => {
                logger.error(format!("Unable to bind socket: {e}"));
                return Error::ERR_CANT_CONNECT;
            }
        };
        if let Err(e) = socket.set_nonblocking(false) {
            logger.error(format!("Unable to set socket as blocking: {e}"));
            return Error::ERR_CANT_CREATE;
        }
        if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs_f32(0.1))) {
            logger.error(format!("Unable to set read timeout for socket: {e}"));
        }
        if let Err(e) = socket.connect(self.ip_address.unwrap()) {
            logger.error(format!(
                "Unable to connect to address {address}: {e}",
                address = self.ip_address.unwrap()
            ));
            return Error::ERR_CANT_CONNECT;
        }

        let (thread_sender, godot_receiver) = mpsc::channel::<Data>();
        let (godot_sender, thread_receiver) = mpsc::channel::<()>();

        let thread_logger = self.logger.bind().clone();
        let mut buf = Vec::with_capacity(1024);
        let handle = std::thread::spawn(move || loop {
            buf.clear();

            if let Ok(_) = thread_receiver.try_recv() {
                break;
            }

            // TODO
            if let Err(e) = socket.send(SEND_DATA.as_bytes()) {
                // if let Err(e) = socket.send_to(SEND_DATA.as_bytes(), ("192.168.88.98", 21412)) {
                thread_logger.error(format!("Unable to send message on socket: {e}"));
            } else {
                thread_logger.debug("sent data");
            }

            match socket.recv(&mut buf) {
                Ok(_) => {
                    let data = match serde_json::from_slice::<InData>(buf.as_slice()) {
                        Ok(v) => v,
                        Err(e) => {
                            thread_logger.error(format!("Error while receiving data: {e}"));
                            continue;
                        }
                    };

                    if let Err(e) = thread_sender.send(Data::from(data)) {
                        thread_logger.error(format!("Error while sending data back to godot: {e}"));
                    } else {
                        godot_print!("sent data!");
                    }
                }
                Err(e) => {
                    thread_logger.error(format!("Unexpected error while receiving: {e}"));
                }
            }
        });

        self.receive_handle = Some(handle);
        self.thread_killer = Some(godot_sender);
        self.receiver = Some(godot_receiver);

        Error::OK
    }

    fn stop(&mut self) -> Error {
        let logger = self.logger.bind();

        if self.receive_handle.is_none() {
            logger.error("Receiver was not started.");
            return Error::ERR_UNAVAILABLE;
        }
        if self.thread_killer.is_none() {
            logger.error("No thread sender found. This is a major bug.");
            return Error::ERR_UNAVAILABLE;
        }

        let thread_killer = self.thread_killer.as_ref().unwrap();
        if let Err(e) = thread_killer.send(()) {
            logger.error(format!("MAJOR BUG: {e}"));
        }

        let handle = self.receive_handle.take().unwrap();
        if let Err(e) = handle.join() {
            logger.error(format!("MAJOR BUG: {e:?}"));
        }

        Error::OK
    }

    fn poll(&mut self) {
        match self.receiver.as_ref().unwrap().try_recv() {
            Ok(v) => {
                godot_print!("{v:?}");
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.logger
                    .bind()
                    .error("Receiver was disconnected somehow, shutting down MeowFace");
                self.stop();
            }
        }
    }

    fn handle_puppet3d(&self, mut puppet: Gd<Puppet3d>) {
        let mut p = puppet.bind_mut();
        p.visit_meow_face(&self.data);
    }

    fn handle_puppet2d(&self, mut puppet: Gd<Puppet2d>) {
        let p = puppet.bind_mut();

        todo!()
    }
}

super::bind_receiver_to_godot!(MeowFace);

impl MeowFace {
    fn new() -> Self {
        Self {
            data: Data::default(),
            logger: Logger::create(gstring!("MeowFace")),

            ip_address: None,
            receive_handle: None,
            thread_killer: None,
            receiver: None,
        }
    }
}
