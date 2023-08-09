use std::path::PathBuf;

use chrono::{serde::ts_seconds, DateTime, Utc};
use godot::{
    engine::{global::Error, ProjectSettings},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::Logger;

/// A newtype that represents a path that Godot is meant to use.
///
/// This can also be used for arbitrary paths, as Godot can handle arbitrary
/// paths as well.
#[derive(Debug, Default, Serialize, Deserialize)]
struct GodotPath(String);

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

#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
pub struct RunnerData {
    /// The name of the Runner Data. Should generally be set to the name of the model.
    name: String,
    /// The path to the runner used for handling the model.
    runner_path: GodotPath,
    /// The path to the gui used in the runner.
    gui_path: GodotPath,
    /// The path to the model
    model_path: GodotPath,
    /// The path to the preview image for the runner.
    preview_path: GodotPath,
    /// Whether the [RunnerData] should be listed as a favorite.
    is_favorite: bool,
    /// The last used time. Used for sorting runners.
    #[serde(with = "ts_seconds")]
    last_used: DateTime<Utc>,
}

#[godot_api]
impl RefCountedVirtual for RunnerData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::new()
    }
}

#[godot_api]
impl RunnerData {
    /// Try to load a `RunnerData` from a given path.
    ///
    /// # Returns
    /// The `RunnerData` if successful or an empty `Variant` otherwise.
    #[func]
    fn try_load(path: GodotString) -> Variant {
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path(path)
            .to_string()
            .into();

        if let Ok(v) = std::fs::read_to_string(&path) {
            if let Ok(v) = tot::from_str::<RunnerData>(v.as_str()) {
                return Gd::new(v).to_variant();
            }
        }

        Logger::global(
            "RunnerData",
            &format!("Unable to load runner data from path {path:?}"),
        );

        Variant::nil()
    }

    /// Try to save the `RunnerData` to the user data directory.
    ///
    /// # Returns
    /// OK on success or an error code otherwise.
    #[func]
    fn try_save(&self) -> Error {
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path(format!("user://{}", self.to_file_name()).into())
            .to_string()
            .into();

        let contents = match tot::to_string(&self) {
            Ok(v) => v,
            Err(e) => {
                Logger::global(
                    "RunnerData",
                    &format!("Unable to convert RunnerData to string: {e}"),
                );

                return Error::ERR_INVALID_DATA;
            }
        };

        match std::fs::write(path, contents) {
            Ok(_) => Error::OK,
            Err(e) => {
                Logger::global("source", &format!("Unable to save RunnerData: {e}"));
                Error::ERR_FILE_CANT_WRITE
            }
        }
    }

    /// Set the `last_used` timestamp to now in UTC time.
    #[func]
    fn timestamp(&mut self) {
        self.last_used = Utc::now();
    }

    #[func]
    fn get_name(&self) -> GodotString {
        self.name.clone().into()
    }

    #[func]
    fn set_name(&mut self, name: GodotString) {
        self.name = name.to_string()
    }

    #[func]
    fn get_runner_path(&self) -> GodotString {
        self.runner_path.clone().into()
    }

    #[func]
    fn set_runner_path(&mut self, runner_path: GodotString) {
        self.runner_path = runner_path.into();
    }

    #[func]
    fn get_gui_path(&self) -> GodotString {
        self.gui_path.clone().into()
    }

    #[func]
    fn set_gui_path(&mut self, gui_path: GodotString) {
        self.gui_path = gui_path.into();
    }

    #[func]
    fn get_model_path(&self) -> GodotString {
        self.model_path.clone().into()
    }

    #[func]
    fn set_model_path(&mut self, model_path: GodotString) {
        self.model_path = model_path.into();
    }

    #[func]
    fn get_preview_path(&self) -> GodotString {
        self.preview_path.clone().into()
    }

    #[func]
    fn set_preview_path(&mut self, preview_path: GodotString) {
        self.preview_path = preview_path.into();
    }

    #[func]
    fn get_is_favorite(&self) -> bool {
        self.is_favorite
    }

    #[func]
    fn set_is_favorite(&mut self, is_favorite: bool) {
        self.is_favorite = is_favorite;
    }

    /// Get the last used date as a string.
    #[func]
    fn get_last_used_string(&self) -> GodotString {
        self.last_used
            .format("%Y/%m/%d %H:%M:%S")
            .to_string()
            .into()
    }

    /// Get the last used date as a unix timestamp.
    #[func]
    fn get_last_used_int(&self) -> i64 {
        self.last_used.timestamp()
    }
}

impl RunnerData {
    /// Create a new [RunnerData].
    fn new() -> Self {
        Self {
            is_favorite: false,
            last_used: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Construct a file name based off of the configured data.
    fn to_file_name(&self) -> String {
        format!("{}.tot", self.name)
    }
}
