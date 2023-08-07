use godot::prelude::*;

use crate::{gstring, vstring, Logger};

#[derive(Debug, GodotClass)]
#[class(base = Node2D)]
pub struct Puppet2d {
    #[var]
    logger: Gd<Logger>,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl Node2DVirtual for Puppet2d {
    fn init(base: godot::obj::Base<Self::Base>) -> Self {
        Self {
            logger: Logger::create(gstring!("Puppet2d")),

            base,
        }
    }

    fn ready(&mut self) {
        // TODO stub
    }
}

#[godot_api]
impl Puppet2d {}
