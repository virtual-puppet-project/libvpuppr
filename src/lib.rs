mod logger;
mod puppet;
mod receiver;

use godot::prelude::*;

struct GodotExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotExtension {}
