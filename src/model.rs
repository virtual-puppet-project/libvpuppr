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

#[derive(Debug, Default, GodotClass)]
#[property(name = name, type = GodotString, get = get_name, set = set_name)]
#[property(name = runner_path, type = GodotString, get = get_runner_path, set = set_runner_path)]
#[property(name = gui_path, type = GodotString, get = get_gui_path, set = set_gui_path)]
#[property(name = model_path, type = GodotString, get = get_model_path, set = set_model_path)]
struct RunnerData {
    data: NewRunnerData,
    #[var]
    id: GodotString,
    #[var]
    preview_path: GodotString,
    #[var]
    is_favorite: bool,
    #[var]
    last_used: Dictionary,
}

impl From<NewRunnerData> for RunnerData {
    fn from(value: NewRunnerData) -> Self {
        Self {
            data: value,
            ..Default::default()
        }
    }
}

#[godot_api]
impl RefCountedVirtual for RunnerData {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::default()
    }
}

#[godot_api]
impl RunnerData {
    #[func]
    fn from_dict(data: Dictionary) -> Option<Gd<RunnerData>> {
        for i in [
            "id",
            "name",
            "runner_path",
            "gui_path",
            "model_path",
            "preview_path",
            "is_favorite",
            "last_used",
        ] {
            if !data.contains_key(i) {
                log::error!("Missing key {i}");
                return None;
            }
        }

        let mut rd = RunnerData::default();
        rd.set_name(GodotString::from_variant(&data.get("name").unwrap()));
        rd.set_runner_path(GodotString::from_variant(&data.get("runner_path").unwrap()));
        rd.set_gui_path(GodotString::from_variant(&data.get("gui_path").unwrap()));
        rd.set_model_path(GodotString::from_variant(&data.get("model_path").unwrap()));
        rd.set_preview_path(GodotString::from_variant(
            &data.get("preview_path").unwrap(),
        ));
        rd.set_is_favorite(bool::from_variant(&data.get("is_favorite").unwrap()));
        rd.set_last_used(Dictionary::from_variant(&data.get("last_used").unwrap()));

        Some(Gd::new(rd))
    }

    #[func]
    fn from_array(data: Array<Variant>) -> Option<Gd<RunnerData>> {
        if data.len() != 8 {
            log::error!("Invalid data, expected 8 values received {}", data.len());
            return None;
        }

        let mut rd = RunnerData::default();
        for (idx, v) in data.iter_shared().enumerate() {
            match idx {
                0 => rd.set_id(GodotString::from_variant(&v)),
                1 => rd.set_name(GodotString::from_variant(&v)),
                2 => rd.set_runner_path(GodotString::from_variant(&v)),
                3 => rd.set_gui_path(GodotString::from_variant(&v)),
                4 => rd.set_model_path(GodotString::from_variant(&v)),
                5 => rd.set_preview_path(GodotString::from_variant(&v)),
                6 => rd.set_is_favorite(bool::from_variant(&v)),
                7 => rd.set_last_used(Dictionary::from_variant(&v)),
                _ => log::error!("Unhandled idx {idx} with data {v}"),
            }
        }

        Some(Gd::new(rd))
    }

    #[func]
    fn get_name(&self) -> GodotString {
        self.data.name.clone()
    }

    #[func]
    fn set_name(&mut self, name: GodotString) {
        self.data.name = name;
    }

    #[func]
    fn get_runner_path(&self) -> GodotString {
        self.data.runner_path.clone()
    }

    #[func]
    fn set_runner_path(&mut self, runner_path: GodotString) {
        self.data.runner_path = runner_path;
    }

    #[func]
    fn get_gui_path(&self) -> GodotString {
        self.data.gui_path.clone()
    }

    #[func]
    fn set_gui_path(&mut self, gui_path: GodotString) {
        self.data.gui_path = gui_path;
    }

    #[func]
    fn get_model_path(&self) -> GodotString {
        self.data.model_path.clone()
    }

    #[func]
    fn set_model_path(&mut self, model_path: GodotString) {
        self.data.model_path = model_path;
    }
}

#[derive(Debug, Default, Clone, GodotClass)]
#[class(init)]
struct NewRunnerData {
    #[var]
    name: GodotString,
    #[var]
    runner_path: GodotString,
    #[var]
    gui_path: GodotString,
    #[var]
    model_path: GodotString,
}

#[godot_api]
impl NewRunnerData {
    #[func]
    fn to_runner_data(&self) -> Gd<RunnerData> {
        Gd::new(RunnerData::from(self.clone()))
    }
}
