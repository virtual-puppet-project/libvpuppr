/*!
A receiver for [MeowFace](https://play.google.com/store/apps/details?id=com.suvidriel.meowface) data.
*/

use godot::{engine::global::Error, prelude::*};
use std::net::{IpAddr, Ipv4Addr};

use super::Receiver;

#[derive(Debug, GodotClass)]
struct MeowFace {
    listen_address: IpAddr,
}

#[godot_api]
impl RefCountedVirtual for MeowFace {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::new()
    }
}

impl Receiver<MeowFace> for MeowFace {
    fn create_inner(data: Dictionary) -> Gd<MeowFace> {
        let mut meow_face = Self::new();
        // TODO stub

        Gd::new(meow_face)
    }

    fn start_inner(data: Dictionary) -> i64 {
        // TODO stub

        -1
    }

    fn stop_inner() -> Error {
        // TODO stub

        Error::OK
    }
}

super::bind_receiver_to_godot!(MeowFace);

impl MeowFace {
    fn new() -> Self {
        Self {
            listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        }
    }
}
