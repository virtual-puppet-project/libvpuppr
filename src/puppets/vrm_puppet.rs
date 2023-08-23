use std::collections::HashMap;

use godot::{
    engine::{global::Error, AnimationPlayer, ArrayMesh, MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{gstring, model::tracking_data::MeowFaceData, Logger};

use super::{BlendShapeMapping, MorphData, Puppet, Puppet3d};

const ANIM_PLAYER: &str = "AnimationPlayer";
const MESH_INST_3D: &str = "MeshInstance3D";
const VRM_META: &str = "vrm_meta";

#[repr(i64)]
#[derive(Debug, Property)]
pub enum VrmType {
    Base = 0,
    PerfectSync = 1,
}

/// Possible ways a VRM model should be handled.
///
/// In theory, all VRM models should be compatible with `Base`, while only some
/// models are compatible with `PerfectSync`. This is due to `PerfectSync` adding
/// additional blend shapes that are not present in the base VRM specification.
#[derive(Debug)]
enum VrmFeatures {
    /// Base VRM 0.0 and 1.0 specification.
    Base {
        left_eye_id: i32,
        right_eye_id: i32,

        expression_data: HashMap<String, Vec<MorphData>>,
    },
    /// Generally refers to an additional 52 blend shapes provided outside
    /// of the VRM specification.
    PerfectSync,
}

impl Default for VrmFeatures {
    fn default() -> Self {
        Self::Base {
            left_eye_id: i32::default(),
            right_eye_id: i32::default(),
            expression_data: HashMap::default(),
        }
    }
}

impl VrmFeatures {
    fn new_base(puppet: &mut VrmPuppet) -> Self {
        let logger = puppet.logger();

        let mut expression_data = HashMap::new();

        let anim_player = match puppet
            .base
            .find_child_ex(ANIM_PLAYER.into())
            .owned(false)
            .done()
        {
            Some(v) => match v.try_cast::<AnimationPlayer>() {
                Some(v) => v,
                None => {
                    logger.error("Unable to cast node to Animation Player, bailing out early!");
                    return Self::default();
                }
            },
            None => {
                logger.error("Unable to find Animation Player, bailing out early!");
                return Self::default();
            }
        };

        for animation_name in anim_player.get_animation_list().as_slice() {
            let animation = match anim_player.get_animation(animation_name.into()) {
                Some(v) => v,
                None => {
                    logger.error("Unable to get animation while setting up, this is a serious bug. Bailing out!");
                    return Self::default();
                }
            };

            let mut morphs = vec![];

            for track_idx in 0..animation.get_track_count() {
                let track_name = animation.track_get_path(track_idx).to_string();
                let (node_name, morph_name) = match track_name.split_once(":") {
                    Some(v) => v,
                    None => {
                        logger.error(format!(
                            "Unable to split track {track_name}, this is slightly unexpected"
                        ));
                        continue;
                    }
                };

                let mesh = match puppet.get_nested_node_or_null(NodePath::from(node_name)) {
                    Some(v) => {
                        if !v.is_class(MESH_INST_3D.into()) {
                            continue;
                        }

                        match v.try_cast::<MeshInstance3D>() {
                            Some(v) => v,
                            None => {
                                logger.error(format!(
                                    "Unable to cast {node_name} to mesh instance, bailing out!"
                                ));
                                return Self::default();
                            }
                        }
                    }
                    None => {
                        logger.error(format!(
                            "Unable to find mesh instance for {node_name}, bailing out!"
                        ));
                        return Self::default();
                    }
                };

                // TODO this is probably unsafe?
                let values = (
                    animation.track_get_key_value(track_idx, 0).to::<f32>(),
                    animation.track_get_key_value(track_idx, 1).to::<f32>(),
                );

                morphs.push(MorphData::new(mesh, morph_name.to_string(), values));
            }

            expression_data.insert(animation_name.to_string(), morphs);
        }

        // TODO find eye id values
        Self::Base {
            left_eye_id: 0,
            right_eye_id: 0,
            expression_data,
        }
    }

    fn new_perfect_sync(_puppet: &mut VrmPuppet) -> Self {
        Self::PerfectSync
    }
}

#[derive(Debug, GodotClass)]
#[class(base = Node3D)]
pub struct VrmPuppet {
    #[var]
    pub logger: Gd<Logger>,

    #[base]
    base: Base<Node3D>,

    #[var]
    blink_threshold: f32,
    #[var]
    link_eye_blinks: bool,
    #[var]
    use_raw_eye_rotation: bool,

    #[var]
    vrm_type: VrmType,
    // Intentionally not exposed
    vrm_features: VrmFeatures,
    #[var]
    vrm_meta: Dictionary,

    #[var]
    pub skeleton: Option<Gd<Skeleton3D>>,
    #[var]
    head_bone: GodotString,
    #[var]
    pub head_bone_id: i32,
    #[var]
    pub additional_movement_bones: Array<i32>,
    #[var]
    initial_bone_poses: Dictionary,

    /// Used for manually manipulating each blend shape.
    blend_shape_mappings: HashMap<String, BlendShapeMapping>,
}

#[godot_api]
impl Node3DVirtual for VrmPuppet {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            logger: Logger::create(gstring!("VrmPuppet")),

            base,

            blink_threshold: 0.0,
            link_eye_blinks: false,
            use_raw_eye_rotation: false,

            vrm_type: VrmType::Base,
            vrm_features: VrmFeatures::default(),
            vrm_meta: Dictionary::new(),

            skeleton: None,
            head_bone: GodotString::new(),
            head_bone_id: -1,
            additional_movement_bones: Array::new(),
            initial_bone_poses: Dictionary::new(),

            blend_shape_mappings: HashMap::new(),
        }
    }

    fn ready(&mut self) {
        let logger = self.logger();

        logger.debug("Starting ready!");

        match self.find_skeleton(&self.base) {
            Some(v) => {
                let _ = self.skeleton.replace(v);
            }
            None => {
                logger.error("Unable to cast to Skeleton3D, bailing out early!");
                return;
            }
        }

        let skeleton = self.skeleton.as_ref().unwrap();

        self.head_bone_id = skeleton.find_bone(self.head_bone.clone());
        if self.head_bone_id < 0 {
            logger.error("No head bone found!");
            return;
        }

        // TODO init skeleton bone transforms from config

        // This must be done after loading the user's custom rest pose
        for i in 0..skeleton.get_bone_count() {
            self.initial_bone_poses.insert(i, skeleton.get_bone_pose(i));
        }

        // Pre-allocate the name here and then clone it in the loop
        let mesh_instance_3d_name = StringName::from(MESH_INST_3D);

        // Populating the blend shape mappings is extremely verbose
        for child in skeleton.get_children().iter_shared() {
            // Used for debugging only
            let child_name = child.get_name();

            if !child.is_class(mesh_instance_3d_name.clone().into()) {
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

        let vrm_meta = match self
            .managed_node()
            .get(VRM_META.into())
            .try_to::<Dictionary>()
        {
            Ok(v) => v,
            Err(e) => {
                logger.error(format!("Unable to get vrm metadata, bailing out! {e:?}"));
                return;
            }
        };
        self.vrm_meta.extend_dictionary(vrm_meta, true);

        self.vrm_features = match self.vrm_type {
            VrmType::Base => VrmFeatures::new_base(self),
            VrmType::PerfectSync => VrmFeatures::new_perfect_sync(self),
        };
    }
}

#[godot_api]
impl VrmPuppet {
    /// Move VRM bones into an a-pose.
    #[func]
    pub fn a_pose(&mut self) -> Error {
        let logger = self.logger();

        let vrm = &self.vrm_meta;

        let mappings = match vrm.get("humanoid_bone_mapping") {
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
                logger.error("No humanoid_bone_mapping found on vrm_meta");
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

    #[func(rename = handle_meow_face)]
    fn handle_meow_face_bound(&mut self, data: Gd<MeowFaceData>) {
        self.handle_meow_face(data)
    }
}

impl Puppet for VrmPuppet {
    fn logger(&self) -> Logger {
        self.logger.bind().clone()
    }

    fn managed_node(&self) -> Gd<Node> {
        match self.base.get_child(0) {
            Some(v) => v,
            None => {
                self.logger()
                    .error("Unable to get managed node, this is a major error!");

                panic!("Bailing out!");
            }
        }
    }
}

impl Puppet3d for VrmPuppet {
    fn handle_meow_face(&mut self, data: Gd<MeowFaceData>) {
        let data = data.bind();
        let skeleton = self.skeleton.as_mut().unwrap();

        // if let Some(position) = data.position {
        //     skeleton.set_bone_pose_position(self.head_bone_id, position);
        // }
        if let Some(rotation) = data.rotation {
            skeleton.set_bone_pose_rotation(
                self.head_bone_id,
                Quaternion::from_euler(Vector3::new(rotation.y, rotation.x, rotation.z) * 0.02),
            );
        }

        match &self.vrm_features {
            VrmFeatures::Base {
                left_eye_id,
                right_eye_id,
                expression_data,
            } => {}
            VrmFeatures::PerfectSync => {}
        }
    }
}
