use godot::prelude::*;
use serde::{Deserialize, Serialize};

use super::Mapper;
use crate::puppets::{puppet_2d::Puppet2d, puppet_3d::Puppet3d};

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    #[serde(rename = "Rotation")]
    rotation: Option<Vector3>,
    #[serde(rename = "Position")]
    position: Option<Vector3>,
    #[serde(rename = "EyeLeft")]
    eye_left: Option<Vector3>,
    #[serde(rename = "EyeRight")]
    eye_right: Option<Vector3>,
    #[serde(rename = "BlendShapes")]
    blend_shapes: Option<Vec<BlendShape>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlendShape {
    k: String,
    v: f32,
}

#[derive(Debug, GodotClass)]
struct MeowFaceMapper;

#[godot_api]
impl RefCountedVirtual for MeowFaceMapper {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self
    }
}

impl super::Mapper for MeowFaceMapper {
    fn handle_puppet3d(data: PackedByteArray, mut puppet: Gd<Puppet3d>) {
        let data = match serde_json::from_slice::<Data>(data.as_slice()) {
            Ok(v) => v,
            Err(e) => {
                godot_error!("{e}");
                return;
            }
        };

        let mut puppet = puppet.bind_mut();
        let head_bone_id = puppet.head_bone_id;

        {
            let skeleton = puppet.skeleton.as_mut().unwrap();
            if let Some(position) = data.position {
                skeleton.set_bone_pose_position(head_bone_id, position);
            }
            if let Some(rotation) = data.rotation {
                skeleton.set_bone_pose_rotation(head_bone_id, Quaternion::from_euler(rotation));
            }
        }
    }

    fn handle_puppet2d(_data: PackedByteArray, _puppet: Gd<Puppet2d>) {
        todo!()
    }
}

super::bind_mapper_to_godot!(MeowFaceMapper);
