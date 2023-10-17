use godot::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Property)]
#[repr(i64)]
pub enum PuppetType {
    None = 0,
    Glb = 1,
    Vrm = 2,
    Png = 3,
}

// TODO workaround until enums can be bound without requiring a struct field
impl From<i64> for PuppetType {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Glb,
            2 => Self::Vrm,
            3 => Self::Png,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum PuppetData {
    #[default]
    None,
    Glb(GlbData),
    Vrm(VrmData),
    Png(PngData),
}

impl PuppetData {
    pub fn glb() -> Self {
        Self::Glb(GlbData::default())
    }

    pub fn vrm() -> Self {
        Self::Vrm(VrmData::default())
    }

    pub fn png() -> Self {
        Self::Png(PngData::default())
    }

    pub fn get_head_bone(&self) -> String {
        match self {
            PuppetData::None => "".into(),
            PuppetData::Glb(v) => v.puppet.head_bone.clone(),
            PuppetData::Vrm(v) => v.puppet.head_bone.clone(),
            PuppetData::Png(_) => "".into(),
        }
    }

    pub fn set_head_bone(&mut self, head_bone: String) {
        match self {
            PuppetData::None => {}
            PuppetData::Glb(v) => v.puppet.head_bone = head_bone,
            PuppetData::Vrm(v) => v.puppet.head_bone = head_bone,
            PuppetData::Png(_) => {}
        }
    }

    // TODO bind IkTargetTransforms

    pub fn get_blink_threshold(&self) -> f32 {
        match self {
            PuppetData::None => 0.0,
            PuppetData::Glb(_) => 0.0,
            PuppetData::Vrm(v) => v.blink_threshold,
            PuppetData::Png(_) => 0.0,
        }
    }

    pub fn set_blink_threshold(&mut self, blink_threshold: f32) {
        match self {
            PuppetData::None => {}
            PuppetData::Glb(_) => {}
            PuppetData::Vrm(v) => v.blink_threshold = blink_threshold,
            PuppetData::Png(_) => {}
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Puppet3d {
    pub head_bone: String,
    pub ik_target_transforms: IkTargetTransforms,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlbData {
    pub puppet: Puppet3d,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VrmData {
    pub puppet: Puppet3d,
    pub blink_threshold: f32,
    pub link_eye_blinks: bool,
    pub use_raw_eye_rotation: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IkTargetTransforms {
    pub head: Transform3D,
    pub left_hand: Transform3D,
    pub right_hand: Transform3D,
    pub hips: Transform3D,
    pub left_foot: Transform3D,
    pub right_foot: Transform3D,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum VrmType {
    #[default]
    Base,
    PerfectSync,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Puppet2d {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PngData {
    pub puppet: Puppet2d,
}
