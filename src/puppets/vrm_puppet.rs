use std::collections::HashMap;

use godot::{
    engine::{
        animation::TrackType, global::Error, AnimationPlayer, ArrayMesh, MeshInstance3D, Skeleton3D,
    },
    prelude::*,
};

use crate::{
    gstring,
    model::{self, tracking_data::MeowFaceData},
    Logger,
};

use super::{BlendShapeMapping, MorphData, Puppet, Puppet3d};

const ANIM_PLAYER: &str = "AnimationPlayer";
const MESH_INST_3D: &str = "MeshInstance3D";
const VRM_META: &str = "vrm_meta";

#[repr(i64)]
#[derive(Debug, Clone, Copy, Property, Export)]
pub enum VrmType {
    Base = 0,
    PerfectSync = 1,
}

impl From<model::puppet::VrmType> for VrmType {
    fn from(value: model::puppet::VrmType) -> Self {
        match value {
            model::puppet::VrmType::Base => Self::Base,
            model::puppet::VrmType::PerfectSync => Self::PerfectSync,
        }
    }
}

impl Into<model::puppet::VrmType> for VrmType {
    fn into(self) -> model::puppet::VrmType {
        match self {
            VrmType::Base => model::puppet::VrmType::Base,
            VrmType::PerfectSync => model::puppet::VrmType::PerfectSync,
        }
    }
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

        let reset_anim = match anim_player.get_animation("RESET".into()) {
            Some(v) => v,
            None => {
                logger.error("Unable to find RESET animation, bailing out!");
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

            for to_track_idx in 0..animation.get_track_count() {
                let track_name = animation.track_get_path(to_track_idx).to_string();
                let (node_name, morph_name) = match track_name.split_once(":") {
                    Some(v) => v,
                    None => {
                        logger.error(format!(
                            "Unable to split track {track_name}, this is slightly unexpected"
                        ));
                        continue;
                    }
                };

                if animation.get_track_count() != 1 {
                    logger.info(format!("Animation {animation_name}:{track_name} does not have exactly 1 key, skipping!"));
                    continue;
                }

                let from_track_idx =
                    reset_anim.find_track(track_name.clone().into(), TrackType::TYPE_BLEND_SHAPE);
                if from_track_idx < 0 {
                    logger.debug(format!(
                        "Reset track does not contain {track_name}, skipping!"
                    ));
                    continue;
                }

                // TODO this seems to be hitting too many false positives
                if animation.track_get_key_count(from_track_idx) < 1 {
                    logger.debug(format!("Reset track does not contain a key, skipping!"));
                    continue;
                }
                if animation.track_get_key_count(to_track_idx) < 1 {
                    logger.debug(format!("{track_name} does not contain a key, skipping!"));
                    continue;
                }

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

                let values = (
                    animation.track_get_key_value(from_track_idx, 0).to::<f32>(),
                    animation.track_get_key_value(to_track_idx, 0).to::<f32>(),
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
// Puppet3d
#[property(name = head_bone, type = GodotString, get = get_head_bone, set = set_head_bone)]
#[property(name = head_bone_id, type = GodotString, get = get_head_bone_id, set = set_head_bone_id)]
#[property(
    name = additional_movement_bones,
    type = Array<i32>,
    get = get_additional_movement_bones,
    set = set_additional_movement_bones,
)]
#[property(
    name = initial_bone_poses,
    type = Dictionary,
    get = get_initial_bone_poses,
    set = set_initial_bone_poses
)]
// VrmPuppet
#[property(name = blink_threshold, type = f32, get = get_blink_threshold, set = set_blink_threshold)]
#[property(name = link_eye_blinks, type = bool, get = get_link_eye_blinks, set = set_link_eye_blinks)]
#[property(name = use_raw_eye_rotation, type = bool, get = get_use_raw_eye_rotation, set = set_use_raw_eye_rotation)]
// #[property(name = vrm_type, type = VrmType, get = get_vrm_type, set = set_vrm_type)]
pub struct VrmPuppet {
    #[var]
    pub logger: Gd<Logger>,

    #[base]
    pub base: Base<Node3D>,

    pub puppet3d: model::puppet::Puppet3d,
    pub vrm_puppet: model::puppet::VrmPuppet,

    // Intentionally not exposed
    vrm_features: VrmFeatures,
    #[var]
    pub vrm_meta: Option<Gd<Resource>>,

    #[var]
    pub skeleton: Option<Gd<Skeleton3D>>,

    /// Used for manually manipulating each blend shape.
    blend_shape_mappings: HashMap<String, BlendShapeMapping>,
}

#[godot_api]
impl Node3DVirtual for VrmPuppet {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            logger: Logger::create(gstring!("VrmPuppet")),

            base,

            puppet3d: model::puppet::Puppet3d::default(),
            vrm_puppet: model::puppet::VrmPuppet::default(),

            vrm_features: VrmFeatures::default(),
            vrm_meta: None,

            skeleton: None,

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

        self.puppet3d.head_bone_id = skeleton.find_bone(self.puppet3d.head_bone.clone().into());
        if self.puppet3d.head_bone_id < 0 {
            logger.error("No head bone found!");
            return;
        }

        // TODO init skeleton bone transforms from config

        // This must be done after loading the user's custom rest pose
        for i in 0..skeleton.get_bone_count() {
            self.puppet3d
                .initial_bone_poses
                .insert(i, skeleton.get_bone_pose(i));
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
            .try_to::<Gd<Resource>>()
        {
            Ok(v) => v,
            Err(e) => {
                logger.error(format!("Unable to get vrm metadata, bailing out! {e:?}"));
                return;
            }
        };
        self.vrm_meta = Some(vrm_meta);

        self.vrm_features = match self.vrm_puppet.vrm_type {
            model::puppet::VrmType::Base => VrmFeatures::new_base(self),
            model::puppet::VrmType::PerfectSync => VrmFeatures::new_perfect_sync(self),
        };

        if self.a_pose() != Error::OK {
            logger.error("Unable to a-pose");
        }
    }
}

#[godot_api]
impl VrmPuppet {
    /// Move VRM bones into an a-pose.
    #[func]
    pub fn a_pose(&mut self) -> Error {
        let logger = self.logger();

        let skeleton = match &mut self.skeleton {
            Some(v) => v,
            None => {
                logger.error("Skeleton was None while trying to a-pose. This is a bug!");
                return Error::ERR_UNCONFIGURED;
            }
        };

        const L_SHOULDER: &str = "LeftShoulder";
        const R_SHOULDER: &str = "RightShoulder";
        const L_UPPER_ARM: &str = "LeftUpperArm";
        const R_UPPER_ARM: &str = "RightUpperArm";

        for bone_name in [L_SHOULDER, R_SHOULDER, L_UPPER_ARM, R_UPPER_ARM] {
            let bone_idx = skeleton.find_bone(bone_name.into());
            if bone_idx < 0 {
                logger.error(format!(
                    "Bone not found while trying to a-pose: {bone_name}"
                ));
                continue;
            }

            let quat = match bone_name {
                L_SHOULDER => {
                    skeleton.get_bone_pose_rotation(bone_idx)
                        * Quaternion::from_angle_axis(Vector3::LEFT, 0.34)
                }

                L_UPPER_ARM => {
                    skeleton.get_bone_pose_rotation(bone_idx)
                        * Quaternion::from_angle_axis(Vector3::RIGHT, 0.52)
                }

                R_SHOULDER => {
                    skeleton.get_bone_pose_rotation(bone_idx)
                        * Quaternion::from_angle_axis(Vector3::LEFT, 0.34)
                }

                R_UPPER_ARM => {
                    skeleton.get_bone_pose_rotation(bone_idx)
                        * Quaternion::from_angle_axis(Vector3::RIGHT, 0.52)
                }

                _ => unreachable!("This should never happen!"),
            };

            skeleton.set_bone_pose_rotation(bone_idx, quat);
        }

        Error::OK
    }

    #[func]
    fn get_head_bone(&self) -> GodotString {
        self.puppet3d.head_bone.clone().into()
    }

    #[func]
    fn set_head_bone(&mut self, head_bone: GodotString) {
        self.puppet3d.head_bone = head_bone.into();
    }

    #[func]
    fn get_head_bone_id(&self) -> i32 {
        self.puppet3d.head_bone_id
    }

    #[func]
    fn set_head_bone_id(&mut self, head_bone_id: i32) {
        self.puppet3d.head_bone_id = head_bone_id
    }

    #[func]
    fn get_additional_movement_bones(&self) -> Array<i32> {
        self.puppet3d
            .additional_movement_bones
            .iter()
            .map(|v| v.clone())
            .collect::<Array<i32>>()
    }

    #[func]
    fn set_additional_movement_bones(&mut self, additional_movement_bones: Array<i32>) {
        self.puppet3d.additional_movement_bones = additional_movement_bones
            .iter_shared()
            .collect::<Vec<i32>>();
    }

    #[func]
    fn get_initial_bone_poses(&self) -> Dictionary {
        self.puppet3d
            .initial_bone_poses
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Dictionary>()
    }

    #[func]
    fn set_initial_bone_poses(&mut self, initial_bone_poses: Dictionary) {
        self.puppet3d.initial_bone_poses = initial_bone_poses
            .iter_shared()
            .map(|(k, v)| {
                (
                    k.try_to::<i32>().unwrap_or_default(),
                    v.try_to::<Transform3D>().unwrap_or_default(),
                )
            })
            .collect::<HashMap<i32, Transform3D>>();
    }

    #[func]
    fn get_blink_threshold(&self) -> f32 {
        self.vrm_puppet.blink_threshold
    }

    #[func]
    fn set_blink_threshold(&mut self, blink_threshold: f32) {
        self.vrm_puppet.blink_threshold = blink_threshold;
    }

    #[func]
    fn get_link_eye_blinks(&self) -> bool {
        self.vrm_puppet.link_eye_blinks
    }

    #[func]
    fn set_link_eye_blinks(&mut self, link_eye_blinks: bool) {
        self.vrm_puppet.link_eye_blinks = link_eye_blinks;
    }

    #[func]
    fn get_use_raw_eye_rotation(&self) -> bool {
        self.vrm_puppet.use_raw_eye_rotation
    }

    #[func]
    fn set_use_raw_eye_rotation(&mut self, use_raw_eye_rotation: bool) {
        self.vrm_puppet.use_raw_eye_rotation = use_raw_eye_rotation;
    }

    // #[func]
    // fn get_vrm_type(&self) -> VrmType {
    //     self.vrm_puppet.vrm_type.into()
    // }

    // #[func]
    // fn set_vrm_type(&mut self, vrm_type: VrmType) {
    //     self.vrm_puppet.vrm_type = vrm_type.into();
    // }

    #[func(rename = handle_meow_face)]
    fn handle_meow_face_bound(&mut self, data: Gd<MeowFaceData>) {
        self.handle_meow_face(data)
    }

    #[func(rename = handle_media_pipe)]
    fn handle_media_pipe_bound(&mut self, projection: Projection, blend_shapes: Dictionary) {
        self.handle_media_pipe(projection, blend_shapes);
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

        if let Some(rotation) = data.rotation {
            skeleton.set_bone_pose_rotation(
                self.puppet3d.head_bone_id,
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

    fn handle_media_pipe(&mut self, projection: Projection, blend_shapes: Dictionary) {
        let skeleton = self.skeleton.as_mut().unwrap();

        let tx = Transform3D::from_projection(projection.inverse());

        skeleton.set_bone_pose_rotation(self.puppet3d.head_bone_id, tx.basis.to_quat());

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
