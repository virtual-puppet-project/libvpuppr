// pub mod puppet_2d;
// pub mod puppet_3d;

pub mod glb_puppet;
pub mod png_puppet;
pub mod vrm_puppet;

use godot::{
    engine::{MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{gstring, model::tracking_data::MeowFaceData, Logger};

pub(crate) trait Puppet {
    fn get_logger(&self) -> Logger;
}

pub const SKELETON_NODE_NAME_3D: &str = "*Skeleton*";
pub(crate) trait Puppet3d: Puppet {
    fn find_skeleton(&self, base: &Base<Node3D>) -> Option<Gd<Skeleton3D>> {
        if let Some(v) = base
            .find_child_ex(gstring!(SKELETON_NODE_NAME_3D))
            .owned(false)
            .done()
        {
            v.try_cast::<Skeleton3D>()
        } else {
            self.get_logger().error("Unable to find skeleton node!");
            None
        }
    }

    fn handle_meow_face(&mut self, data: Gd<MeowFaceData>);
}

/// Contains data necessary for manipulating blend shapes.
#[derive(Debug)]
pub(crate) struct BlendShapeMapping {
    /// The mesh the blend shape is associated with.
    mesh: Gd<MeshInstance3D>,
    /// The property path to the blend shape.
    blend_shape_path: String,
    /// The value of the blend shape, generally from 0.0-1.0.
    value: f32,
}

impl BlendShapeMapping {
    pub(crate) fn new(mesh: Gd<MeshInstance3D>, blend_shape_path: String, value: f32) -> Self {
        Self {
            mesh,
            blend_shape_path,
            value,
        }
    }
}

pub(crate) trait Puppet2d: Puppet {}
