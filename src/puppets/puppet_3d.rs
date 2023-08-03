use std::collections::HashMap;

use godot::{
    engine::{global::Error, ArrayMesh, MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{gstring, vstring, Logger};

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
#[derive(Debug, GodotClass)]
struct VrmData {
    /// VRM metadata stored in the `gltf` model.
    vrm_meta: Dictionary,

    /// VRM expressions can be mapped to multiple blend shapes.
    expression_mappings: HashMap<String, Vec<BlendShapeMapping>>,

    /// The specific way a model should be handled based off of its features.
    vrm_features: VrmFeatures,
}

#[godot_api]
impl RefCountedVirtual for VrmData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            vrm_meta: Dictionary::new(),
            expression_mappings: HashMap::new(),
            vrm_features: VrmFeatures::None,
        }
    }
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
const SKELETON_NODE_NAME: &str = "Skeleton3D";

/// A 3D puppet, compatible with both regular `glb` models and `vrm` models.
#[derive(Debug, GodotClass)]
#[class(base = Node3D)]
struct Puppet3d {
    /// The [Logger] for the puppet.
    #[var]
    logger: Gd<Logger>,

    /// The base Godot node this struct inherits from.
    #[base]
    base: Base<Node3D>,

    /// Whether the puppet should try and load vrm-specific fields.
    #[var]
    is_vrm: bool,
    #[var]
    vrm_data: Option<Gd<VrmData>>,

    /// The skeleton of the puppet.
    #[var]
    skeleton: Option<Gd<Skeleton3D>>,
    /// The name of the head bone. This is only used to find the `head_bone_id`.
    #[var]
    head_bone: GodotString,
    /// The index/id of the head bone in the skeleton.
    #[var]
    head_bone_id: i32,
    /// Additional bone ids that should be moved when the head bone moves.
    #[var]
    additional_movement_bones: Array<i32>,
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
        if let Some(v) = self.base.find_child(gstring!(SKELETON_NODE_NAME)) {
            match v.try_cast::<Skeleton3D>() {
                Some(v) => {
                    let _ = self.skeleton.replace(v);
                }
                None => {
                    self.logger
                        .bind()
                        .error(vstring!("Unable to find skeleton node, bailing out early!"));
                    return;
                }
            }
        }

        let skeleton = self.skeleton.as_ref().unwrap();

        self.head_bone_id = skeleton.find_bone(self.head_bone.clone());
        if self.head_bone_id < 0 {
            self.logger.bind().error(vstring!("No head bone found!"));
        }

        // TODO init skeleton bone transforms from config

        // This must be done after loading the user's custom rest pose
        for i in 0..skeleton.get_bone_count() {
            self.initial_bone_poses.insert(i, skeleton.get_bone_pose(i));
        }

        // Populating the blend shape mappings is extremely verbose
        for child in skeleton.get_children().iter_shared() {
            if !child.is_class(gstring!("MeshInstance3D")) {
                continue;
            }

            let child = child.try_cast::<MeshInstance3D>();
            if child.is_none() {
                self.logger.bind().error(vstring!(
                    "Skeleton child was a MeshInstance3D but was unable to cast to MeshInstance3D"
                ));
                continue;
            }

            let child = child.unwrap();
            let mesh = match child.get_mesh() {
                Some(v) => v,
                None => {
                    self.logger
                        .bind()
                        .error(vstring!("Unable to get mesh from MeshInstance3D, skipping"));
                    continue;
                }
            };
            let mesh = match mesh.try_cast::<ArrayMesh>() {
                Some(v) => v,
                None => {
                    self.logger
                        .bind()
                        .error(vstring!("Unable to convert mesh into ArrayMesh, skipping"));
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
    }
}

#[godot_api]
impl Puppet3d {
    /// Move VRM bones into an a-pose.
    #[func]
    fn a_pose(&mut self) -> Error {
        let logger = self.logger.bind();

        if !self.is_vrm {
            logger.warn(vstring!(
                "A VRM model is required for automatic a-posing. This is because VRM models guarantee certain bones exist."
            ));
            return Error::ERR_UNCONFIGURED;
        }
        if self.vrm_data.is_none() {
            logger.error(vstring!("vrm_data is None, this is a bug!"));
            return Error::ERR_INVALID_DATA;
        }

        let vrm = self.vrm_data.as_ref().unwrap().bind();

        let mappings = match vrm.vrm_meta.get("humanoid_bone_mapping") {
            Some(v) => {
                if v.get_type() != VariantType::Dictionary {
                    logger.error(vstring!("humanoid_bone_mapping was not a Dictionary"));
                    return Error::ERR_INVALID_DATA;
                }
                match v.try_to::<Dictionary>() {
                    Ok(v) => v,
                    Err(_) => {
                        logger.error(vstring!("Unable to convert humanoid_bone_mapping Variant to Dictionary, this is probably a godot-rust bug!"));
                        return Error::ERR_INVALID_DATA;
                    }
                }
            }
            None => {
                self.logger
                    .bind()
                    .error(vstring!("No humanoid_bone_mapping found on vrm_meta"));
                return Error::ERR_INVALID_DATA;
            }
        };

        let skeleton = match &mut self.skeleton {
            Some(v) => v,
            None => {
                logger.error(vstring!(
                    "Skeleton was None while trying to a-pose. This is a bug!"
                ));
                return Error::ERR_UNCONFIGURED;
            }
        };

        const L_SHOULDER: &str = "leftShoulder";
        const R_SHOULDER: &str = "rightShoulder";
        const L_UPPER_ARM: &str = "leftUpperArm";
        const R_UPPER_ARM: &str = "rightUpperArm";

        for bone_name in [L_SHOULDER, R_SHOULDER, L_UPPER_ARM, R_UPPER_ARM] {
            if !mappings.contains_key(bone_name) {
                logger.error(vstring!(format!("humanoid_bone_mapping does not contain bone while trying to a-pose: {bone_name}")));
                continue;
            }

            let bone_idx = skeleton.find_bone(bone_name.into());
            if bone_idx < 0 {
                logger.error(vstring!(format!(
                    "Bone not found while trying to a-pose: {bone_name}"
                )));
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
}
