mod logger;
pub use logger::Logger;
pub mod model;
mod puppets;
mod receivers;

use godot::prelude::*;

struct GodotExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotExtension {}
