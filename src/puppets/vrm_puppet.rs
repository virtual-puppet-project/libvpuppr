use std::collections::{hash_map::RandomState, HashMap};

use godot::{
    engine::{
        animation::TrackType, global::Error, AnimationPlayer, ArrayMesh, MeshInstance3D, Skeleton3D,
    },
    prelude::*,
};
use log::{debug, error};
use rayon::prelude::*;

use crate::{
    model::{
        self,
        puppet::{PuppetData, VrmData},
        tracking_data::VTubeStudioData,
        IFacialMocapData,
    },
    Logger,
};

use super::{BlendShapeMapping, IkTargets3d, Puppet, Puppet3d, Puppet3dError};

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
    Base { left_eye_id: i32, right_eye_id: i32 },
    /// Generally refers to an additional 52 blend shapes provided outside
    /// of the VRM specification.
    PerfectSync,
}

impl Default for VrmFeatures {
    fn default() -> Self {
        Self::Base {
            left_eye_id: i32::default(),
            right_eye_id: i32::default(),
        }
    }
}

#[derive(Debug, GodotClass)]
#[class(base = Node3D)]
pub struct VrmPuppet {
    #[var]
    pub logger: Gd<Logger>,

    #[base]
    pub base: Base<Node3D>,

    // Intentionally not exposed
    vrm_features: VrmFeatures,
    #[var]
    pub vrm_meta: Option<Gd<Resource>>,

    #[var]
    pub skeleton: Option<Gd<Skeleton3D>>,
    #[var]
    pub ik_targets_3d: Option<Gd<IkTargets3d>>,

    blend_shape_mappings: HashMap<String, BlendShapeMapping>,
    expression_mappings: HashMap<String, Vec<String>>,
}

#[godot_api]
impl Node3DVirtual for VrmPuppet {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            logger: Logger::create("VrmPuppet".into()),

            base,

            vrm_features: VrmFeatures::default(),
            vrm_meta: None,

            skeleton: None,
            ik_targets_3d: None,

            blend_shape_mappings: HashMap::new(),
            expression_mappings: HashMap::new(),
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
                logger.error("Unable to find skeleton, bailing out early!");
                return;
            }
        }

        let skeleton = self.skeleton.as_ref().unwrap();

        // TODO init skeleton bone transforms from config

        // This must be done after loading the user's custom rest pose
        // for i in 0..skeleton.get_bone_count() {
        //     self.puppet3d
        //         .initial_bone_poses
        //         .insert(i, skeleton.get_bone_pose(i));
        // }

        let mut ik_targets_3d = IkTargets3d::default();
        // TODO these are all hardcoded, maybe pull values from elsewhere?
        if let v @ Some(_) = self.create_armature("HeadArmature", "Head") {
            ik_targets_3d.head = v;

            let mut tx = skeleton.get_bone_global_pose(skeleton.find_bone("Head".into()));
            tx.origin = self.base.to_global(tx.origin);
            ik_targets_3d.head_starting_transform = tx;
        }
        if let v @ Some(_) = self.create_armature("LeftHandArmature", "LeftHand") {
            ik_targets_3d.left_hand = v;

            let mut tx = skeleton.get_bone_global_pose(skeleton.find_bone("LeftHand".into()));
            tx.origin = self.base.to_global(tx.origin);
            ik_targets_3d.left_hand_starting_transform = tx;
        }
        if let v @ Some(_) = self.create_armature("RightHandArmature", "RightHand") {
            ik_targets_3d.right_hand = v;

            let mut tx = skeleton.get_bone_global_pose(skeleton.find_bone("RightHand".into()));
            tx.origin = self.base.to_global(tx.origin);
            ik_targets_3d.right_hand_starting_transform = tx;
        }
        if let v @ Some(_) = self.create_armature("HipsArmature", "Hips") {
            ik_targets_3d.hips = v;
        }
        if let v @ Some(_) = self.create_armature("LeftFootArmature", "LeftFoot") {
            ik_targets_3d.left_foot = v;
        }
        if let v @ Some(_) = self.create_armature("RightFootArmature", "RightFoot") {
            ik_targets_3d.right_foot = v;
        }
        self.ik_targets_3d = Some(Gd::new(ik_targets_3d));

        populate_blend_shape_mappings(&mut self.blend_shape_mappings, skeleton);
        if let Some(v) = self.find_animation_player() {
            populate_and_modify_expression_mappings(&mut self.expression_mappings, &v);
        } else {
            error!("Unable to find Animation Player, blend shapes will not work!");
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

        // self.vrm_features = match self.vrm_puppet.vrm_type {
        //     model::puppet::VrmType::Base => VrmFeatures::new_base(self),
        //     model::puppet::VrmType::PerfectSync => VrmFeatures::new_perfect_sync(self),
        // };

        // if self.a_pose() != Error::OK {
        //     logger.error("Unable to a-pose");
        // }
    }
}

// TODO does Godot guarantee unique names for autogenerated blend shape names?
/// Iterate through every child node of a [Skeleton3D] and, if that node is a
/// [MeshInstance3D], register every blend shape present on the mesh.
fn populate_blend_shape_mappings(
    // mappings: &mut Arc<RwLock<HashMap<String, BlendShapeMapping>>>,
    mappings: &mut HashMap<String, BlendShapeMapping>,
    skeleton: &Gd<Skeleton3D>,
) {
    let mesh_instance_3d_name = StringName::from(MESH_INST_3D);
    // let mut mappings = mappings.write().unwrap();

    for child in skeleton.get_children().iter_shared() {
        // Used for debugging only
        let child_name = child.get_name();

        if !child.is_class(mesh_instance_3d_name.clone().into()) {
            debug!("Child {child_name} was not a MeshInstance3D, skipping");
            continue;
        }

        let child = child.try_cast::<MeshInstance3D>();
        if child.is_none() {
            error!(
                "Skeleton child {child_name} was a MeshInstance3D but was unable to cast to MeshInstance3D");
            continue;
        }

        let child = child.unwrap();
        let mesh = match child.get_mesh() {
            Some(v) => v,
            None => {
                error!("Unable to get mesh from MeshInstance3D {child_name}, skipping");
                continue;
            }
        };
        let mesh = match mesh.try_cast::<ArrayMesh>() {
            Some(v) => v,
            None => {
                error!("Unable to convert mesh from {child_name} into ArrayMesh, skipping");
                continue;
            }
        };

        for i in 0..mesh.get_blend_shape_count() {
            let blend_shape_name = mesh.get_blend_shape_name(i).to_string();
            let blend_shape_property_path = format!("blend_shapes/{}", blend_shape_name);
            let value = child.get_blend_shape_value(i);

            let instance_id = child.instance_id().to_i64();
            // Quick sanity check to make sure instance ids are valid
            if let None = InstanceId::try_from_i64(instance_id)
                .map(|v| Gd::<MeshInstance3D>::try_from_instance_id(v))
            {
                error!(
                    "Invalid instance id for {}::{blend_shape_name}, skipping!",
                    child.get_name()
                );
                continue;
            }

            mappings.insert(
                blend_shape_name.clone(),
                BlendShapeMapping::new(instance_id, blend_shape_property_path, value),
            );
        }
    }
}

/// Extract VRM and Perfect Sync mappings from the godot-vrm [AnimationPlayer].
/// Each mapping is a [String] name to a list of blend shape mapping keys.
///
/// Mapping names are converted to lowercase, since naming for expressions is
/// extremely inconsistent.
fn populate_and_modify_expression_mappings(
    mappings: &mut HashMap<String, Vec<String>>,
    anim_player: &Gd<AnimationPlayer>,
) {
    let valid_track_types = [TrackType::TYPE_ROTATION_3D, TrackType::TYPE_BLEND_SHAPE];

    for animation_name in anim_player.get_animation_list().as_slice() {
        let animation = match anim_player.get_animation(animation_name.into()) {
            Some(v) => v,
            None => {
                error!(
                    "Unable to get animation while setting up, this is a serious bug. Bailing out!",
                );
                return;
            }
        };

        let mut morphs = vec![];

        for track_idx in 0..animation.get_track_count() {
            let track_name = animation.track_get_path(track_idx).to_string();
            let track_type = animation.track_get_type(track_idx);
            if !valid_track_types.contains(&track_type) {
                debug!("{track_name} is not handled, skipping");
                continue;
            }

            let (_node_name, morph_name) = match track_name.split_once(":") {
                Some(v) => v,
                None => {
                    error!("Unable to split track {track_name}, this is slightly unexpected");
                    continue;
                }
            };

            if animation.track_get_key_count(track_idx) < 1 {
                debug!("{track_name} does not contain a key, skipping!");
                continue;
            }

            match track_type {
                TrackType::TYPE_ROTATION_3D => {
                    debug!("rotation tracks not yet handled");
                }
                TrackType::TYPE_BLEND_SHAPE => morphs.push(morph_name.to_string()),
                _ => {
                    error!(
                        "Trying to handle invalid track type {track_type:?}, this is a major bug!"
                    )
                }
            }
        }

        mappings.insert(animation_name.to_string().to_lowercase(), morphs);
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

    #[func(rename = handle_i_facial_mocap)]
    fn handle_i_facial_mocap_bound(&mut self, data: Gd<IFacialMocapData>) {
        self.handle_i_facial_mocap(data);
    }

    #[func(rename = handle_vtube_studio)]
    fn handle_vtube_studio_bound(&mut self, data: Gd<VTubeStudioData>) {
        self.handle_vtube_studio(data);
    }

    #[func(rename = handle_meow_face)]
    fn handle_meow_face_bound(&mut self, data: Gd<VTubeStudioData>) {
        self.handle_meow_face(data)
    }

    #[func(rename = handle_media_pipe)]
    fn handle_media_pipe_bound(&mut self, projection: Projection, blend_shapes: Dictionary) {
        self.handle_media_pipe(projection, blend_shapes);
    }
}

impl VrmPuppet {
    fn find_animation_player(&self) -> Option<Gd<AnimationPlayer>> {
        if let Some(v) = self
            .base
            .find_child_ex(ANIM_PLAYER.into())
            .owned(false)
            .done()
        {
            v.try_cast::<AnimationPlayer>()
        } else {
            None
        }
    }

    fn create_armature(&self, armature_name: &str, bone_name: &str) -> Option<Gd<Node3D>> {
        let skeleton = self.skeleton.as_ref().unwrap();

        let bone_idx = skeleton.find_bone(bone_name.into());
        if bone_idx < 0 {
            return None;
        }

        let mut tx = skeleton.get_bone_global_pose(bone_idx);
        tx.origin = self.base.to_global(tx.origin);

        let mut armature = Node3D::new_alloc();
        armature.set_name(armature_name.into());
        armature.set_transform(tx);

        Some(armature)
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

// NOTE we're using a slight hack to apply blend shapes as fast as possible
// gdext classes are not Sync, but as long as they are created/destroyed in the
// same thread, they can be used. Thus, we can find the mesh instance from the
// instance id and modify it in the thread
//
// This does mean that the code is extremely not DRY
impl Puppet3d for VrmPuppet {
    // fn init_pose(&mut self, data: Gd<RunnerData>) -> Result<(), Puppet3dError> {
    //     if self.skeleton.is_none() {
    //         return Err(Puppet3dError::NodeNotReady);
    //     }
    //     let PuppetData::Vrm(VrmData { puppet, .. }) = &data.bind().puppet_data else {
    //         return Err(Puppet3dError::PuppetTypeMismatch);
    //     };

    //     let skeleton = self.skeleton.as_ref().unwrap();

    //     let mut ik_targets_3d = IkTargets3d::default();
    //     // TODO these are all hardcoded, maybe pull values from elsewhere?
    //     if let v @ Some(_) = self.create_armature("HeadArmature", "Head") {
    //         ik_targets_3d.head = v;

    //         let mut tx = skeleton.get_bone_global_pose(skeleton.find_bone("Head".into()));
    //         tx.origin = self.base.to_global(tx.origin);
    //         ik_targets_3d.head_starting_transform = tx;
    //     }
    //     if let v @ Some(_) = self.create_armature("LeftHandArmature", "LeftHand") {
    //         ik_targets_3d.left_hand = v;

    //         let mut tx = skeleton.get_bone_global_pose(skeleton.find_bone("LeftHand".into()));
    //         tx.origin = self.base.to_global(tx.origin);
    //         ik_targets_3d.left_hand_starting_transform = tx;
    //     }
    //     if let v @ Some(_) = self.create_armature("RightHandArmature", "RightHand") {
    //         ik_targets_3d.right_hand = v;

    //         let mut tx = skeleton.get_bone_global_pose(skeleton.find_bone("RightHand".into()));
    //         tx.origin = self.base.to_global(tx.origin);
    //         ik_targets_3d.right_hand_starting_transform = tx;
    //     }
    //     if let v @ Some(_) = self.create_armature("HipsArmature", "Hips") {
    //         ik_targets_3d.hips = v;
    //     }
    //     if let v @ Some(_) = self.create_armature("LeftFootArmature", "LeftFoot") {
    //         ik_targets_3d.left_foot = v;
    //     }
    //     if let v @ Some(_) = self.create_armature("RightFootArmature", "RightFoot") {
    //         ik_targets_3d.right_foot = v;
    //     }
    //     self.ik_targets_3d = Some(Gd::new(ik_targets_3d));

    //     //

    //     Ok(())
    // }

    fn handle_i_facial_mocap(&mut self, data: Gd<IFacialMocapData>) {
        let data = data.bind();
        let skeleton = self.skeleton.as_mut().unwrap();

        if let Some(ik) = self.ik_targets_3d.as_mut() {
            let rotation =
                Vector3::new(data.rotation.x, data.rotation.y, data.rotation.z).to_variant();
            if let Some(v) = ik.bind_mut().head.as_mut() {
                v.call_deferred("set_rotation_degrees".into(), &[rotation.clone()]);
            }
            let mut ik = ik.bind_mut();

            let head_origin = ik.head_starting_transform.origin;
            if let Some(v) = ik.head.as_mut() {
                v.call_deferred(
                    "set_position".into(),
                    &[(head_origin + (data.position)).to_variant()],
                );
            }
            let left_hand_origin = ik.left_hand_starting_transform.origin;
            if let Some(v) = ik.left_hand.as_mut() {
                v.call_deferred(
                    "set_position".into(),
                    &[(left_hand_origin + (data.position)).to_variant()],
                );
            }
            let right_hand_origin = ik.right_hand_starting_transform.origin;
            if let Some(v) = ik.right_hand.as_mut() {
                v.call_deferred(
                    "set_position".into(),
                    &[(right_hand_origin + (data.position)).to_variant()],
                );
            }
        }
        data.blend_shapes.par_iter().for_each(|(k, v)| {
            if let Some(mappings) = self.expression_mappings.get(&k.to_lowercase()) {
                for mapping in mappings {
                    if let Some(mapping) = self.blend_shape_mappings.get(mapping) {
                        Gd::<MeshInstance3D>::from_instance_id(InstanceId::from_i64(
                            mapping.mesh_id,
                        ))
                        .set_indexed(NodePath::from(&mapping.blend_shape_path), v.to_variant());
                    }
                }
            }
        });

        match &self.vrm_features {
            VrmFeatures::Base {
                left_eye_id,
                right_eye_id,
            } => {}
            VrmFeatures::PerfectSync => {}
        }
    }

    fn handle_vtube_studio(&mut self, data: Gd<VTubeStudioData>) {
        let data = data.bind();
        let skeleton = self.skeleton.as_mut().unwrap();

        if let Some(rotation) = data.rotation {
            if let Some(ik) = self.ik_targets_3d.as_mut() {
                let mut ik = ik.bind_mut();

                // Data comes in Unity ordering I think?
                let rotation = Vector3::new(rotation.y, rotation.x, rotation.z);

                let head_rotation = ik.head_starting_transform.basis.to_euler(EulerOrder::YXZ);
                if let Some(v) = ik.head.as_mut() {
                    v.call_deferred(
                        "set_rotation_degrees".into(),
                        &[(rotation - head_rotation).to_variant()],
                    );
                }
            }
        }
        if let Some(position) = data.position {
            if let Some(ik) = self.ik_targets_3d.as_mut() {
                let mut ik = ik.bind_mut();

                let head_origin = ik.head_starting_transform.origin;
                if let Some(v) = ik.head.as_mut() {
                    v.call_deferred(
                        "set_position".into(),
                        &[(head_origin - (position * 0.02)).to_variant()],
                    );
                }

                let left_hand_origin = ik.left_hand_starting_transform.origin;
                if let Some(v) = ik.left_hand.as_mut() {
                    v.call_deferred(
                        "set_position".into(),
                        &[(left_hand_origin - (position * 0.02)).to_variant()],
                    );
                }

                let right_hand_origin = ik.right_hand_starting_transform.origin;
                if let Some(v) = ik.right_hand.as_mut() {
                    v.call_deferred(
                        "set_position".into(),
                        &[(right_hand_origin - (position * 0.02)).to_variant()],
                    );
                }
            }
        }
        if let Some(blend_shapes) = &data.blend_shapes {
            blend_shapes.par_iter().for_each(|v| {
                if let Some(mappings) = self.expression_mappings.get(&v.k.to_lowercase()) {
                    for mapping in mappings {
                        if let Some(mapping) = self.blend_shape_mappings.get(mapping) {
                            Gd::<MeshInstance3D>::from_instance_id(InstanceId::from_i64(
                                mapping.mesh_id,
                            ))
                            .set_indexed(
                                NodePath::from(&mapping.blend_shape_path),
                                v.v.to_variant(),
                            );
                        }
                    }
                }
            });
        }

        match &self.vrm_features {
            VrmFeatures::Base {
                left_eye_id,
                right_eye_id,
            } => {}
            VrmFeatures::PerfectSync => {}
        }
    }

    fn handle_meow_face(&mut self, data: Gd<VTubeStudioData>) {
        self.handle_vtube_studio(data);
    }

    fn handle_media_pipe(&mut self, projection: Projection, blend_shapes: Dictionary) {
        let skeleton = self.skeleton.as_mut().unwrap();

        let tx = Transform3D::from_projection(projection.inverse());

        // skeleton.set_bone_pose_rotation(self.puppet3d.head_bone_id, tx.basis.to_quat());

        let blend_shapes: HashMap<String, f32, RandomState> = HashMap::from_iter(
            blend_shapes
                .iter_shared()
                .map(|(k, v)| (k.to_string(), v.to::<f32>())),
        );

        blend_shapes.par_iter().for_each(|(name, value)| {
            if let Some(mappings) = self.expression_mappings.get(&name.to_lowercase()) {
                for mapping in mappings {
                    if let Some(mapping) = self.blend_shape_mappings.get(mapping) {
                        Gd::<MeshInstance3D>::from_instance_id(InstanceId::from_i64(
                            mapping.mesh_id,
                        ))
                        .set_indexed(
                            NodePath::from(&mapping.blend_shape_path),
                            value.to_variant(),
                        );
                    }
                }
            }
        });

        match &self.vrm_features {
            VrmFeatures::Base {
                left_eye_id,
                right_eye_id,
            } => {}
            VrmFeatures::PerfectSync => {}
        }
    }
}
