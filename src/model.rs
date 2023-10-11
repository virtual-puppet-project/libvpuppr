pub mod puppet;
pub mod tracking_data;

use godot::prelude::*;
use serde::{Deserialize, Serialize};

pub use tracking_data::*;

/// A newtype that represents a path that Godot is meant to use.
///
/// This can also be used for arbitrary paths, as Godot can handle arbitrary
/// paths as well.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GodotPath(String);

impl std::ops::Deref for GodotPath {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<GodotString> for GodotPath {
    fn from(value: GodotString) -> Self {
        Self(value.to_string())
    }
}

impl Into<GodotString> for GodotPath {
    fn into(self) -> GodotString {
        self.0.into()
    }
}
