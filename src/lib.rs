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

/// Helper struct for information about the libvpuppr library.
#[derive(Debug, Default, GodotClass)]
struct LibVpuppr;

#[godot_api]
impl RefCountedVirtual for LibVpuppr {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self
    }
}

#[godot_api]
impl LibVpuppr {
    /// A mapping of various vpuppr metadata.
    #[func]
    fn metadata() -> Dictionary {
        let mut mapping = Dictionary::new();

        let is_debug = if cfg!(debug_assertions) { true } else { false };
        mapping.insert("DEBUG", is_debug);
        mapping.insert("RELEASE", !is_debug);

        mapping.insert("VERSION", env!("CARGO_PKG_VERSION"));
        mapping.insert("VERSION_MAJOR", env!("CARGO_PKG_VERSION_MAJOR"));
        mapping.insert("VERSION_MINOR", env!("CARGO_PKG_VERSION_MINOR"));
        mapping.insert("VERSION_PATCH", env!("CARGO_PKG_VERSION_PATCH"));

        mapping.insert("LIBVPUPPR_AUTHORS", env!("CARGO_PKG_AUTHORS"));

        mapping
    }
}

struct GodotExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotExtension {}
