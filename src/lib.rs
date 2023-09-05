mod logger;
pub mod model;
mod puppets;

use godot::{engine::global::Error, prelude::*};
use log::LevelFilter;

pub use logger::Logger;

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
    /// Initialize logging of Rust libraries.
    ///
    /// # Note
    /// A new [String] must be allocated when printing, otherwise Godot is not
    /// able to print anything.
    #[func]
    fn init_rust_log() -> Error {
        match youlog::Youlog::new_from_default_env()
            .global_level(LevelFilter::Debug)
            .log_fn(LevelFilter::Info, |r| {
                Logger::global(LevelFilter::Info, r.target(), r.args().to_string().as_str());
            })
            .log_fn(LevelFilter::Warn, |r| {
                godot_warn!("{}", r.args().to_string().as_str());
                Logger::global(LevelFilter::Warn, r.target(), r.args().to_string().as_str());
            })
            .log_fn(LevelFilter::Error, |r| {
                godot_error!("{}", r.args().to_string().as_str());
                Logger::global(
                    LevelFilter::Error,
                    r.target(),
                    r.args().to_string().as_str(),
                );
            })
            .log_fn(LevelFilter::Debug, |r| {
                Logger::global(
                    LevelFilter::Debug,
                    r.target(),
                    r.args().to_string().as_str(),
                );
            })
            .init()
        {
            Ok(_) => Error::OK,
            Err(_) => Error::ERR_UNCONFIGURED,
        }
    }

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
