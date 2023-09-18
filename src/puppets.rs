pub mod glb_puppet;
pub mod png_puppet;
pub mod vrm_puppet;

use std::fmt::Display;

use godot::{
    engine::{MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{
    gstring,
    model::tracking_data::{IFacialMocapData, VTubeStudioData},
    Logger,
};

#[derive(Debug, Default, GodotClass)]
#[class(init)]
pub struct IkTargets3d {
    #[var]
    pub head: Option<Gd<Node3D>>,
    #[var]
    pub head_starting_transform: Transform3D,
    #[var]
    pub left_hand: Option<Gd<Node3D>>,
    #[var]
    pub left_hand_starting_transform: Transform3D,
    #[var]
    pub right_hand: Option<Gd<Node3D>>,
    #[var]
    pub right_hand_starting_transform: Transform3D,
    #[var]
    pub hips: Option<Gd<Node3D>>,
    #[var]
    pub left_foot: Option<Gd<Node3D>>,
    #[var]
    pub right_foot: Option<Gd<Node3D>>,
}

#[godot_api]
impl IkTargets3d {}

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

    fn handle_i_facial_mocap(&mut self, data: Gd<IFacialMocapData>);

    fn handle_vtube_studio(&mut self, data: Gd<VTubeStudioData>);

    fn handle_meow_face(&mut self, data: Gd<VTubeStudioData>);

    // TODO you-win Sept 10, 2023: Godot is not able to send GDMP types over the wire
    fn handle_media_pipe(&mut self, projection: Projection, blend_shapes: Dictionary);
}

/// Contains data necessary for manipulating blend shapes. Meant to be viewable by a user.
#[derive(Debug)]
pub struct BlendShapeMapping {
    /// The mesh id of the mesh the blend shape is associated with.
    mesh_id: i64,
    /// The property path to the blend shape.
    blend_shape_path: String,
    /// The value of the blend shape, generally from 0.0-1.0. Is modified in place.
    value: f32,
}

impl BlendShapeMapping {
    pub fn new(mesh_id: i64, blend_shape_path: String, value: f32) -> Self {
        Self {
            mesh_id,
            blend_shape_path,
            value,
        }
    }
}

pub trait Puppet2d: Puppet {}
