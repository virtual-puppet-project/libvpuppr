pub mod glb_puppet;
pub mod png_puppet;
pub mod vrm_puppet;

use godot::{
    engine::{MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{gstring, model::tracking_data::MeowFaceData, Logger};

pub trait Puppet {
    fn logger(&self) -> Logger;
    fn managed_node(&self) -> Gd<Node>;
}

pub const SKELETON_NODE_NAME_3D: &str = "*Skeleton*";
pub trait Puppet3d: Puppet {
    fn find_skeleton(&self, base: &Base<Node3D>) -> Option<Gd<Skeleton3D>> {
        if let Some(v) = base
            .find_child_ex(gstring!(SKELETON_NODE_NAME_3D))
            .owned(false)
            .done()
        {
            v.try_cast::<Skeleton3D>()
        } else {
            self.logger().error("Unable to find skeleton node!");
            None
        }
    }

    fn get_nested_node_or_null(&self, node_path: NodePath) -> Option<Gd<Node>> {
        self.managed_node().get_node_or_null(node_path)
    }

    fn handle_meow_face(&mut self, data: Gd<MeowFaceData>);
}

/// Contains data necessary for manipulating blend shapes. Meant to be viewable by a user.
#[derive(Debug)]
pub struct BlendShapeMapping {
    /// The mesh the blend shape is associated with.
    mesh: Gd<MeshInstance3D>,
    /// The property path to the blend shape.
    blend_shape_path: String,
    /// The value of the blend shape, generally from 0.0-1.0. Is modified in place.
    value: f32,
}

impl BlendShapeMapping {
    pub fn new(mesh: Gd<MeshInstance3D>, blend_shape_path: String, value: f32) -> Self {
        Self {
            mesh,
            blend_shape_path,
            value,
        }
    }
}

#[derive(Debug)]
pub struct MorphData {
    /// The mesh the morph is associated with.
    mesh: Gd<MeshInstance3D>,
    /// The property path to the blend shape.
    blend_shape_path: String,
    // Min/max values for the morph. Is never modified.
    values: (f32, f32),
}

impl MorphData {
    pub fn new(mesh: Gd<MeshInstance3D>, blend_shape_path: String, values: (f32, f32)) -> Self {
        Self {
            mesh,
            blend_shape_path,
            values,
        }
    }
}

pub trait Puppet2d: Puppet {}
