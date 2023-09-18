pub mod puppet;
pub mod tracking_data;

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use chrono::{serde::ts_seconds, DateTime, Utc};
use godot::{
    engine::{global::Error, ProjectSettings},
    prelude::*,
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

pub use tracking_data::*;

#[derive(Debug)]
enum SaveFileError {
    ConversionError { data_name: String },
    ReadError { path: PathBuf },
    WriteError { data_name: String, path: PathBuf },
    FileDoesNotExist { path: PathBuf },
}

impl Display for SaveFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveFileError::ConversionError { data_name } => {
                write!(f, "Failed to convert {data_name} to String")
            }
            SaveFileError::ReadError { path } => {
                write!(f, "Failed to read data from path {path:?}")
            }
            SaveFileError::WriteError { data_name, path } => {
                write!(f, "Failed to write {data_name} to {path:?}")
            }
            SaveFileError::FileDoesNotExist { path } => {
                write!(f, "File does not exist at {path:?}")
            }
        }
    }
}

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

trait SaveFile: Sized {
    fn file_name(&self) -> String;

    fn try_save(&self, path: &PathBuf) -> Result<(), SaveFileError>;

    fn try_load(path: &PathBuf) -> Result<Self, SaveFileError>;
}

/// App-level metadata.
#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
pub struct Metadata {
    /// Absolute paths to runner data files.
    #[serde(default)]
    known_runner_data: Vec<PathBuf>,

    /// Options used when starting iFacialMocap.
    #[serde(default)]
    ifm_options: IfmOptions,
    /// Options used when starting MediaPipe.
    #[serde(default)]
    media_pipe_options: MediaPipeOptions,
    /// Options used when starting VTubeStudio.
    #[serde(default)]
    vtube_studio_options: VTubeStudioOptions,
    /// Options used when starting MeowFace.
    #[serde(default)]
    meow_face_options: MeowFaceOptions,
}

#[godot_api]
impl RefCountedVirtual for Metadata {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::default()
    }
}

#[godot_api]
impl Metadata {
    #[func(rename = try_save)]
    fn try_save_bound(&self) -> Error {
        // TODO use trait methods
        // self.try_save(self.file_name())
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path(format!("user://{}", "metadata.tot").into())
            .to_string()
            .into();

        let contents = match tot::to_string(&self) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to convert RunnerData to string: {e}");

                return Error::ERR_INVALID_DATA;
            }
        };

        match std::fs::write(path, contents) {
            Ok(_) => Error::OK,
            Err(e) => {
                error!("Unable to save RunnerData: {e}");
                Error::ERR_FILE_CANT_WRITE
            }
        }
    }

    #[func]
    fn try_load() -> Variant {
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path("user://metadata.tot".into())
            .to_string()
            .into();

        if let Ok(v) = std::fs::read_to_string(&path) {
            if let Ok(v) = tot::from_str::<Metadata>(v.as_str()) {
                return Gd::new(v).to_variant();
            }
        }

        error!("Unable to load runner data from path {path:?}");

        Variant::nil()
    }

    #[func]
    pub fn scan(&mut self, path: GodotString) -> Error {
        debug!("Scanning for data");

        let path = ProjectSettings::singleton()
            .globalize_path(path)
            .to_string();
        let path = Path::new(&path);

        info!("Scanning for data at path {path:?}");

        let mut found_files = vec![];
        match std::fs::read_dir(path) {
            Ok(v) => {
                for entry in v.into_iter() {
                    if let Ok(entry) = entry {
                        let file_name = entry
                            .file_name()
                            .to_str()
                            .unwrap_or_default()
                            .to_lowercase();
                        if Path::new(&file_name).extension().unwrap_or_default() == "tot" {
                            debug!("Found file {file_name}");
                            found_files.push(path.join(file_name));
                        }
                    }
                }
            }
            Err(e) => {
                error!("{e}");
                return Error::ERR_CANT_OPEN;
            }
        }

        self.known_runner_data = found_files;

        info!("Finished scanning!");

        Error::OK
    }

    /// Tries to load and return all known [RunnerData].
    #[func]
    fn get_known_runner_data(&mut self) -> Array<Gd<RunnerData>> {
        let mut runner_data = Array::new();
        let mut missing_data = vec![];

        for path in self.known_runner_data.iter() {
            let data = match RunnerData::try_load(path) {
                Ok(v) => v,
                Err(e) => {
                    error!("{e}");

                    missing_data.push(path.clone());

                    continue;
                }
            };

            runner_data.push(Gd::new(data));
        }

        self.known_runner_data
            .retain(|v| !missing_data.contains(&v));

        runner_data
    }
}

impl SaveFile for Metadata {
    fn file_name(&self) -> String {
        "metadata.tot".into()
    }

    fn try_save(&self, path: &PathBuf) -> Result<(), SaveFileError> {
        let contents = match tot::to_string(&self) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to convert RunnerData to string: {e}");

                return Err(SaveFileError::ConversionError {
                    data_name: self.file_name(),
                });
            }
        };

        std::fs::write(path, contents).map_err(|_| SaveFileError::WriteError {
            data_name: self.file_name(),
            path: path.to_path_buf(),
        })
    }

    fn try_load(path: &PathBuf) -> Result<Self, SaveFileError> {
        if let Ok(v) = std::fs::read_to_string(&path) {
            return tot::from_str::<Metadata>(v.as_str()).map_err(|_| SaveFileError::ReadError {
                path: path.to_path_buf(),
            });
        }

        error!("Unable to load runner data from path {path:?}");

        Err(SaveFileError::ReadError {
            path: path.to_path_buf(),
        })
    }
}

/// Data for a runner.
#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
#[property(name = name, type = GodotString, get = get_name, set = set_name)]
#[property(name = puppet_class, type = GodotString, get = get_puppet_class, set = set_puppet_class)]
#[property(name = runner_path, type = GodotString, get = get_runner_path, set = set_runner_path)]
#[property(name = gui_path, type = GodotString, get = get_gui_path, set = set_gui_path)]
#[property(name = model_path, type = GodotString, get = get_model_path, set = set_model_path)]
#[property(name = preview_path, type = GodotString, get = get_preview_path, set = set_preview_path)]
#[property(name = is_favorite, type = GodotString, get = get_is_favorite, set = set_is_favorite)]
#[property(name = last_used, type = GodotString, get = get_last_used_int)]
pub struct RunnerData {
    /// The name of the Runner Data. Should generally be set to the name of the model.
    name: String,
    /// The Godot class name of the puppet to use. This is later instantiated via [`ClassDB`].
    puppet_class: String,
    /// The path to the runner used for handling the model.
    runner_path: GodotPath,
    /// The path to the gui used in the runner.
    gui_path: GodotPath,
    /// The path to the model
    model_path: GodotPath,
    /// The path to the preview image for the runner.
    preview_path: GodotPath,
    /// Whether the `RunnerData` should be listed as a favorite.
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
    #[func(rename = try_load)]
    fn try_load_bound(path: GodotString) -> Variant {
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path(path)
            .to_string()
            .into();

        match RunnerData::try_load(&path) {
            Ok(v) => Gd::new(v).to_variant(),
            Err(e) => {
                error!("{e}");
                Variant::nil()
            }
        }
    }

    /// Try to save the `RunnerData` to the user data directory.
    ///
    /// # Returns
    /// OK on success or an error code otherwise.
    #[func(rename = try_save)]
    fn try_save_bound(&self) -> Error {
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path(format!("user://{}", self.file_name()).into())
            .to_string()
            .into();

        match self.try_save(&path) {
            Ok(_) => Error::OK,
            Err(e) => {
                error!("{e}");
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
    fn get_puppet_class(&self) -> GodotString {
        self.puppet_class.clone().into()
    }

    #[func]
    fn set_puppet_class(&mut self, puppet_class: GodotString) {
        self.puppet_class = puppet_class.into();
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
}

impl SaveFile for RunnerData {
    fn file_name(&self) -> String {
        format!("{}.tot", self.name)
    }

    fn try_save(&self, path: &PathBuf) -> Result<(), SaveFileError> {
        let contents = match tot::to_string(&self) {
            Ok(v) => v,
            Err(e) => {
                error!("Unable to convert RunnerData to string: {e}");

                return Err(SaveFileError::ConversionError {
                    data_name: self.file_name(),
                });
            }
        };

        match std::fs::write(path, contents) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Unable to save RunnerData: {e}");
                Err(SaveFileError::WriteError {
                    data_name: self.file_name(),
                    path: path.clone(),
                })
            }
        }
    }

    fn try_load(path: &PathBuf) -> Result<Self, SaveFileError> {
        if let Ok(v) = std::fs::read_to_string(path) {
            if let Ok(v) = tot::from_str::<RunnerData>(v.as_str()) {
                return Ok(v);
            }
        }

        error!("Unable to load runner data from path {path:?}");

        Err(SaveFileError::FileDoesNotExist { path: path.clone() })
    }
}
