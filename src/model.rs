use std::path::PathBuf;

use chrono::{serde::ts_seconds, DateTime, Utc};
use godot::{engine::ProjectSettings, prelude::*};
use serde::{Deserialize, Serialize};

type GodotPath = String;

#[derive(Debug, Default, GodotClass, Serialize, Deserialize)]
pub struct RunnerData {
    name: String,
    runner_path: GodotPath,
    gui_path: GodotPath,
    preview_path: GodotPath,
    is_favorite: bool,
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
    #[func]
    fn try_load(path: GodotString) -> Gd<RunnerData> {
        let path: PathBuf = ProjectSettings::singleton()
            .globalize_path(path)
            .to_string()
            .into();

        if let Ok(v) = std::fs::read_to_string(&path) {
            if let Ok(v) = tot::from_str::<RunnerData>(v.as_str()) {
                return Gd::new(v);
            }
        }

        crate::Logger::global(
            "RunnerData".into(),
            GodotString::from(format!("Unable to load runner data from path {path:?}"))
                .to_variant(),
        );

        Gd::new_default()
    }

    #[func]
    fn try_save(&self) -> u32 {
        0
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
        self.runner_path = runner_path.to_string();
    }

    #[func]
    fn get_gui_path(&self) -> GodotString {
        self.gui_path.clone().into()
    }

    #[func]
    fn set_gui_path(&mut self, gui_path: GodotString) {
        self.gui_path = gui_path.to_string()
    }

    #[func]
    fn get_preview_path(&self) -> GodotString {
        self.preview_path.clone().into()
    }

    #[func]
    fn set_preview_path(&mut self, preview_path: GodotString) {
        self.preview_path = preview_path.to_string();
    }

    #[func]
    fn get_is_favorite(&self) -> bool {
        self.is_favorite
    }

    #[func]
    fn set_is_favorite(&mut self, is_favorite: bool) {
        self.is_favorite = is_favorite;
    }

    #[func]
    fn get_last_used_string(&self) -> GodotString {
        self.last_used
            .format("%Y/%m/%d %H:%M:%S")
            .to_string()
            .into()
    }

    #[func]
    fn get_last_used_int(&self) -> i64 {
        self.last_used.timestamp()
    }
}

impl RunnerData {
    fn new() -> Self {
        Self {
            is_favorite: false,
            last_used: chrono::Utc::now(),
            ..Default::default()
        }
    }
}
