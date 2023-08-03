mod logger;
pub use logger::Logger;
pub mod model;
mod puppets;
mod receivers;

use godot::prelude::*;

/// Easy [GodotString] creation. :lenny:
macro_rules! gstring {
    ($string:expr) => {
        GodotString::from($string)
    };
}
pub(crate) use gstring;

/// Easy [GodotString] as a [Variant] creation.
macro_rules! vstring {
    ($string:expr) => {
        gstring!($string).to_variant()
    };
}
pub(crate) use vstring;

struct GodotExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotExtension {}
