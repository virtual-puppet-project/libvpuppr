use std::collections::HashMap;

use godot::prelude::Transform3D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Puppet3d {
    pub head_bone: String,
    pub head_bone_id: i32,
    pub additional_movement_bones: Vec<i32>,
    pub initial_bone_poses: HashMap<i32, Transform3D>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VrmPuppet {
    pub blink_threshold: f32,
    pub link_eye_blinks: bool,
    pub use_raw_eye_rotation: bool,
    pub vrm_type: VrmType,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum VrmType {
    #[default]
    Base,
    PerfectSync,
}
