use std::collections::HashMap;

use godot::{
    engine::{ArrayMesh, MeshInstance3D, Skeleton3D},
    prelude::*,
};

use crate::{
    model::{
        puppet::{GlbData, PuppetData},
        tracking_data::{IFacialMocapData, VTubeStudioData},
    },
    Logger,
};

use super::{BlendShapeMapping, Puppet, Puppet3d, Puppet3dError};

// TODO this is used in both vrm and glb puppet
const MESH_INST_3D: &str = "MeshInstance3D";

#[derive(Debug, GodotClass)]
#[class(base = Node3D)]
pub struct GlbPuppet {
    #[var]
    pub logger: Gd<Logger>,

    #[base]
    base: Base<Node3D>,

    #[var]
    pub skeleton: Option<Gd<Skeleton3D>>,
    #[var]
    pub head_bone: GodotString,
    #[var]
    pub head_bone_id: i32,
    #[var]
    pub additional_movement_bones: Array<i32>,
    #[var]
    pub initial_bone_poses: Dictionary,

    blend_shape_mappings: HashMap<String, BlendShapeMapping>,
}

#[godot_api]
impl Node3DVirtual for GlbPuppet {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            logger: Logger::create("GlbPuppet".into()),

            base,

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
                        child.instance_id().to_i64(),
                        blend_shape_property_path,
                        value,
                    ),
                );
            }
        }
    }
}

#[godot_api]
impl GlbPuppet {
    #[func(rename = handle_vtube_studio)]
    fn handle_vtube_studio_bound(&mut self, data: Gd<VTubeStudioData>) {
        self.handle_vtube_studio(data);
    }

    #[func(rename = handle_meow_face)]
    fn handle_meow_face_bound(&mut self, data: Gd<VTubeStudioData>) {
        self.handle_meow_face(data);
    }

    #[func(rename = handle_media_pipe)]
    fn handle_media_pipe_bound(&mut self, projection: Projection, blend_shapes: Dictionary) {
        self.handle_media_pipe(projection, blend_shapes);
    }

    #[func(rename = handle_i_facial_mocap)]
    fn handle_i_facial_mocap_bound(&mut self, data: Gd<IFacialMocapData>) {
        self.handle_i_facial_mocap(data);
    }
}

impl Puppet for GlbPuppet {
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

impl Puppet3d for GlbPuppet {
    // fn init_pose(&mut self, data: Gd<RunnerData>) -> Result<(), Puppet3dError> {
    //     let PuppetData::Glb(GlbData { puppet, .. }) = &data.bind().puppet_data else {
    //         return Err(Puppet3dError::PuppetTypeMismatch);
    //     };

    //     // TODO stub

    //     Ok(())
    // }

    fn handle_i_facial_mocap(&mut self, data: Gd<IFacialMocapData>) {
        //
    }

    fn handle_vtube_studio(&mut self, data: Gd<VTubeStudioData>) {
        let data = data.bind();
        let skeleton = self.skeleton.as_mut().unwrap();

        if let Some(rotation) = data.rotation {
            skeleton.set_bone_pose_rotation(
                self.head_bone_id,
                Quaternion::from_euler(Vector3::new(rotation.y, rotation.x, rotation.z) * 0.02),
            );
        }
    }

    fn handle_meow_face(&mut self, data: Gd<VTubeStudioData>) {
        self.handle_vtube_studio(data);
    }

    fn handle_media_pipe(&mut self, projection: Projection, _blend_shapes: Dictionary) {
        let skeleton = self.skeleton.as_mut().unwrap();

        let tx = Transform3D::from_projection(projection);

        skeleton.set_bone_pose_rotation(self.head_bone_id, tx.basis.to_quat());
    }
}
