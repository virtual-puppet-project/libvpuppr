use std::collections::HashMap;

use godot::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};

use super::GodotPath;

#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
#[class(init)]
pub struct IFacialMocapOptions {
    pub address: GodotPath,
    pub port: i32,
}

#[godot_api]
impl IFacialMocapOptions {}

#[derive(Debug, Default, GodotClass)]
pub struct IFacialMocapData {
    pub position: Vector3,
    pub rotation: Vector3,
    pub right_eye: Vector3,
    pub left_eye: Vector3,
    pub blend_shapes: HashMap<String, f32>,
}

#[godot_api]
impl RefCountedVirtual for IFacialMocapData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::default()
    }
}

#[godot_api]
impl IFacialMocapData {
    #[func]
    fn from(data: PackedByteArray) -> Gd<IFacialMocapData> {
        Gd::new(match std::str::from_utf8(data.as_slice()) {
            Ok(v) => {
                let mut r = Self::default();

                let mut split = v.split("|");
                while let Some(v) = split.next() {
                    if let Some((k, v)) = v.split_once('#') {
                        // TODO these are all gross, there must be a better way
                        match k {
                            "=head" => {
                                let vals = v.splitn(5, ',').collect::<Vec<&str>>();

                                r.rotation.x = vals
                                    .get(0)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.rotation.y = vals
                                    .get(1)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.rotation.z = vals
                                    .get(2)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();

                                r.position.x = vals
                                    .get(3)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.position.y = vals
                                    .get(4)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.position.z = vals
                                    .get(5)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                            }
                            "rightEye" => {
                                let vals = v.splitn(2, ',').collect::<Vec<&str>>();

                                r.right_eye.x = vals
                                    .get(0)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.right_eye.y = vals
                                    .get(1)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.right_eye.z = vals
                                    .get(2)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                            }
                            "leftEye" => {
                                let vals = v.splitn(2, ',').collect::<Vec<&str>>();

                                r.left_eye.x = vals
                                    .get(0)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.left_eye.y = vals
                                    .get(1)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                                r.left_eye.z = vals
                                    .get(2)
                                    .map(|v| v.parse::<f32>().unwrap_or_default())
                                    .unwrap_or_default();
                            }
                            _ => error!("Unhandled ifm data key: {k}"),
                        }
                    } else if let Some((k, v)) = v.split_once("-") {
                        r.blend_shapes.insert(
                            k
                                // TODO maybe use https://github.com/BurntSushi/aho-corasick for faster replace?
                                .replace("_L", "left")
                                .replace("_R", "right"),
                            100.0 / v.parse().unwrap_or(0.0),
                        );
                    } else if v.is_empty() {
                    } else {
                        error!("Unhandled ifm key-value pair {v}");
                    }
                }

                r
            }
            Err(e) => {
                error!("{e}");
                Self::default()
            }
        })
    }
}

#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
#[class(init)]
pub struct VTubeStudioOptions {
    pub address: GodotPath,
    pub port: i32,
}

#[godot_api]
impl VTubeStudioOptions {}

#[derive(Debug, Default, Serialize, Deserialize, GodotClass)]
pub struct VTubeStudioData {
    #[serde(rename = "Rotation")]
    pub rotation: Option<Vector3>,
    #[serde(rename = "Position")]
    pub position: Option<Vector3>,
    #[serde(rename = "EyeLeft")]
    pub eye_left: Option<Vector3>,
    #[serde(rename = "EyeRight")]
    pub eye_right: Option<Vector3>,
    #[serde(rename = "BlendShapes")]
    pub blend_shapes: Option<Vec<VtBlendShape>>,
}

#[godot_api]
impl RefCountedVirtual for VTubeStudioData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::default()
    }
}

#[godot_api]
impl VTubeStudioData {
    #[func]
    fn from(data: PackedByteArray) -> Gd<VTubeStudioData> {
        Gd::new(
            match serde_json::from_slice::<VTubeStudioData>(data.as_slice()) {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");
                    Self::default()
                }
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VtBlendShape {
    pub k: String,
    pub v: f32,
}

#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
#[class(init)]
pub struct MeowFaceOptions {
    pub address: GodotPath,
    pub port: i32,
}

#[godot_api]
impl MeowFaceOptions {}

#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
#[class(init)]
pub struct MediaPipeOptions {
    pub camera_resolution: Vector2i,
}

#[godot_api]
impl MediaPipeOptions {}
