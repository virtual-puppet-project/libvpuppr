use godot::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Logger;

trait TrackingData<T: Default> {
    fn from_bytes(_data: PackedByteArray) -> T {
        Logger::global("TrackingData", "from not yet implemented");
        T::default()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, GodotClass)]
#[class(base = RefCounted)]
pub struct MeowFaceData {
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
impl RefCountedVirtual for MeowFaceData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::default()
    }
}

#[godot_api]
impl MeowFaceData {
    #[func]
    fn from(data: PackedByteArray) -> Gd<MeowFaceData> {
        Gd::new(Self::from_bytes(data))
    }
}

impl TrackingData<MeowFaceData> for MeowFaceData {
    fn from_bytes(data: PackedByteArray) -> MeowFaceData {
        match serde_json::from_slice::<MeowFaceData>(data.as_slice()) {
            Ok(v) => v,
            Err(e) => {
                godot_error!("{e}");
                Self::default()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VtBlendShape {
    pub k: String,
    pub v: f32,
}
