mod meow_face;

use godot::prelude::*;

use crate::puppets::{puppet_2d::Puppet2d, puppet_3d::Puppet3d};

trait Mapper {
    fn handle_puppet3d(data: PackedByteArray, puppet: Gd<Puppet3d>);

    fn handle_puppet2d(data: PackedByteArray, puppet: Gd<Puppet2d>);
}

macro_rules! bind_mapper_to_godot {
    ($name:ident) => {
        #[godot_api]
        impl $name {
            #[func(rename = handle_puppet3d)]
            fn handle_puppet3d_bound(&self, data: PackedByteArray, puppet: Gd<Puppet3d>) {
                Self::handle_puppet3d(data, puppet);
            }

            #[func(rename = handle_puppet2d)]
            fn visit_puppet2d_bound(&self, data: PackedByteArray, puppet: Gd<Puppet2d>) {
                Self::handle_puppet2d(data, puppet);
            }
        }
    };
}
pub(crate) use bind_mapper_to_godot;
