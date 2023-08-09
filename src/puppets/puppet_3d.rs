use std::collections::HashMap;

use godot::{
    engine::{global::Error, ArrayMesh, MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{gstring, vstring, Logger};

use super::Visitor;

/// Contains data necessary for manipulating blend shapes.
#[derive(Debug)]
struct BlendShapeMapping {
    /// The mesh the blend shape is associated with.
    mesh: Gd<MeshInstance3D>,
    /// The property path to the blend shape.
    blend_shape_path: String,
    /// The value of the blend shape, generally from 0.0-1.0.
    value: f32,
}

impl BlendShapeMapping {
    fn new(mesh: Gd<MeshInstance3D>, blend_shape_path: String, value: f32) -> Self {
        Self {
            mesh,
            blend_shape_path,
            value,
        }
    }
}

/// VRM-specific data.
#[derive(Debug)]
struct VrmData {
    /// VRM metadata stored in the `gltf` model.
    vrm_meta: Dictionary,

    /// VRM expressions can be mapped to multiple blend shapes.
    expression_mappings: HashMap<String, Vec<BlendShapeMapping>>,

    /// The specific way a model should be handled based off of its features.
    vrm_features: VrmFeatures,
}

/// Possible ways a VRM model should be handled.
///
/// In theory, all VRM models should be compatible with `Base`, while only some
/// models are compatible with `PerfectSync`. This is due to `PerfectSync` adding
/// additional blend shapes that are not present in the base VRM specification.
#[derive(Debug, PartialEq)]
enum VrmFeatures {
    /// No VRM features. This should _not_ be reachable, as Godot should simply
    /// store the associated field as `null`.
    None,
    /// Base VRM 0.0 and 1.0 specification.
    Base {
        left_eye_id: i32,
        right_eye_id: i32,

        blink_threshold: f32,
        link_eye_blinks: bool,
        use_raw_eye_rotation: bool,
    },
    /// Generally refers to an additional 52 blend shapes provided outside
    /// of the VRM specification.
    PerfectSync {},
}

// TODO this might be wrong
/// The default skeleton name for finding the skeleton node.
const SKELETON_NODE_NAME: &str = "*Skeleton*";

/// A 3D puppet, compatible with both regular `glb` models and `vrm` models.
#[derive(Debug, GodotClass)]
#[class(base = Node3D)]
pub struct Puppet3d {
    /// The [Logger] for the puppet.
    #[var]
    pub logger: Gd<Logger>,

    /// The base Godot node this struct inherits from.
    #[base]
    base: Base<Node3D>,

    /// Whether the puppet should try and load vrm-specific fields.
    #[var]
    pub is_vrm: bool,
    vrm_data: Option<VrmData>,

    /// The skeleton of the puppet.
    #[var]
    pub skeleton: Option<Gd<Skeleton3D>>,
    /// The name of the head bone. This is only used to find the `head_bone_id`.
    #[var]
    head_bone: GodotString,
    /// The index/id of the head bone in the skeleton.
    #[var]
    pub head_bone_id: i32,
    /// Additional bone ids that should be moved when the head bone moves.
    #[var]
    pub additional_movement_bones: Array<i32>,
    /// The initial pose of the skeleton for easy pose resetting.
    #[var]
    initial_bone_poses: Dictionary,

    /// Internal mapping of blend shapes. Used for directly accessing blend shape data.
    blend_shape_mappings: HashMap<String, BlendShapeMapping>,
}

#[godot_api]
impl Node3DVirtual for Puppet3d {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            logger: Logger::create(gstring!("Puppet3d")),

            base,

            is_vrm: false,
            vrm_data: None,

            skeleton: None,
            head_bone: GodotString::new(),
            head_bone_id: -1,
            additional_movement_bones: Array::new(),
            initial_bone_poses: Dictionary::new(),

            blend_shape_mappings: HashMap::new(),
        }
    }

    fn ready(&mut self) {
        let logger = self.logger.bind();

        logger.debug("Starting ready!");

        if let Some(v) = self
            .base
            .find_child_ex(gstring!(SKELETON_NODE_NAME))
            .owned(false)
            .done()
        {
            match v.try_cast::<Skeleton3D>() {
                Some(v) => {
                    let _ = self.skeleton.replace(v);
                }
                None => {
                    logger.error("Unable to cast to skeleton node, bailing out early!");
                    return;
                }
            }
        } else {
            logger.error("Unable to find skeleton node, bailing out early!");
            return;
        }

        let skeleton = self.skeleton.as_ref().unwrap();

        self.head_bone_id = skeleton.find_bone(self.head_bone.clone());
        if self.head_bone_id < 0 {
            logger.error("No head bone found!");
        }

        // TODO init skeleton bone transforms from config

        // This must be done after loading the user's custom rest pose
        for i in 0..skeleton.get_bone_count() {
            self.initial_bone_poses.insert(i, skeleton.get_bone_pose(i));
        }

        // Pre-allocate the name here and then clone it in the loop
        let mesh_instance_3d_name = gstring!("MeshInstance3D");

        // Populating the blend shape mappings is extremely verbose
        for child in skeleton.get_children().iter_shared() {
            // Used for debugging only
            let child_name = child.get_name();

            if !child.is_class(mesh_instance_3d_name.clone()) {
                logger.debug(format!(
                    "Child {child_name} was not a MeshInstance3D, skipping"
                ));
                continue;
            }

            let child = child.try_cast::<MeshInstance3D>();
            if child.is_none() {
                logger.error(
                    format!("Skeleton child {child_name} was a MeshInstance3D but was unable to cast to MeshInstance3D")
                );
                continue;
            }

            let child = child.unwrap();
            let mesh = match child.get_mesh() {
                Some(v) => v,
                None => {
                    logger.error(format!(
                        "Unable to get mesh from MeshInstance3D {child_name}, skipping"
                    ));
                    continue;
                }
            };
            let mesh = match mesh.try_cast::<ArrayMesh>() {
                Some(v) => v,
                None => {
                    logger.error(format!(
                        "Unable to convert mesh from {child_name} into ArrayMesh, skipping"
                    ));
                    continue;
                }
            };

            for i in 0..mesh.get_blend_shape_count() {
                let blend_shape_name = mesh.get_blend_shape_name(i).to_string();
                let blend_shape_property_path = format!("blend_shapes/{}", blend_shape_name);
                let value = child.get_blend_shape_value(i);

                self.blend_shape_mappings.insert(
                    blend_shape_name.clone(),
                    BlendShapeMapping::new(
                        // TODO this seems strange
                        Gd::from_instance_id(child.instance_id()),
                        blend_shape_property_path,
                        value,
                    ),
                );
            }
        }

        if self.is_vrm {
            //
        }
    }
}

#[godot_api]
impl Puppet3d {
    #[func]
    pub fn set_vrm_meta(&mut self, vrm_meta: Dictionary) {
        match self.vrm_data.as_mut() {
            Some(v) => {
                v.vrm_meta = vrm_meta;

                // TODO stub
            }
            None => {
                self.logger.bind().error(
                    "Tried to set vrm_meta on a non-vrm model or the VrmData struct was None.",
                );
            }
        }
    }

    /// Move VRM bones into an a-pose.
    #[func]
    pub fn a_pose(&mut self) -> Error {
        let logger = self.logger.bind();

        if !self.is_vrm {
            logger.warn(
                "A VRM model is required for automatic a-posing. This is because VRM models guarantee certain bones exist."
            );
            return Error::ERR_UNCONFIGURED;
        }
        if self.vrm_data.is_none() {
            logger.error("vrm_data is None, this is a bug!");
            return Error::ERR_INVALID_DATA;
        }

        let vrm = self.vrm_data.as_ref().unwrap();

        let mappings = match vrm.vrm_meta.get("humanoid_bone_mapping") {
            Some(v) => {
                if v.get_type() != VariantType::Dictionary {
                    logger.error("humanoid_bone_mapping was not a Dictionary");
                    return Error::ERR_INVALID_DATA;
                }
                match v.try_to::<Dictionary>() {
                    Ok(v) => v,
                    Err(_) => {
                        logger.error("Unable to convert humanoid_bone_mapping Variant to Dictionary, this is probably a godot-rust bug!");
                        return Error::ERR_INVALID_DATA;
                    }
                }
            }
            None => {
                self.logger
                    .bind()
                    .error("No humanoid_bone_mapping found on vrm_meta");
                return Error::ERR_INVALID_DATA;
            }
        };

        let skeleton = match &mut self.skeleton {
            Some(v) => v,
            None => {
                logger.error("Skeleton was None while trying to a-pose. This is a bug!");
                return Error::ERR_UNCONFIGURED;
            }
        };

        const L_SHOULDER: &str = "leftShoulder";
        const R_SHOULDER: &str = "rightShoulder";
        const L_UPPER_ARM: &str = "leftUpperArm";
        const R_UPPER_ARM: &str = "rightUpperArm";

        for bone_name in [L_SHOULDER, R_SHOULDER, L_UPPER_ARM, R_UPPER_ARM] {
            if !mappings.contains_key(bone_name) {
                logger.error(format!("humanoid_bone_mapping does not contain bone while trying to a-pose: {bone_name}"));
                continue;
            }

            let bone_idx = skeleton.find_bone(bone_name.into());
            if bone_idx < 0 {
                logger.error(format!(
                    "Bone not found while trying to a-pose: {bone_name}"
                ));
                continue;
            }

            let quat = match bone_name {
                L_SHOULDER => Quaternion::new(0.0, 0.0, 0.1, 0.85),
                R_SHOULDER => Quaternion::new(0.0, 0.0, -0.1, 0.85),
                L_UPPER_ARM => Quaternion::new(0.0, 0.0, 0.4, 0.85),
                R_UPPER_ARM => Quaternion::new(0.0, 0.0, -0.4, 0.85),
                _ => unreachable!("This should never happen!"),
            };
            skeleton.set_bone_pose_rotation(bone_idx, quat);
        }

        Error::OK
    }

    #[func(rename = visit_meow_face)]
    fn visit_meow_face_bound(&mut self, meow_face: Gd<crate::receivers::meow_face::MeowFace>) {
        self.visit_meow_face(&meow_face.bind().data);
    }
}

impl super::Visitor for Puppet3d {
    fn visit_mediapipe(&mut self, _data: godot::prelude::Dictionary) {
        //
    }

    fn visit_meow_face(&mut self, data: &crate::receivers::meow_face::Data) {
        let skeleton = self.skeleton.as_mut().unwrap();
        skeleton.set_bone_pose_position(self.head_bone_id, data.head_position);
        skeleton.set_bone_pose_rotation(
            self.head_bone_id,
            Quaternion::from_euler(data.head_rotation),
        );
    }
}
