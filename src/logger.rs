use std::io::Write;

use godot::{engine::ProjectSettings, prelude::*};

const MAX_LOGS: u8 = 100;
static mut LOG_STORE: Vec<String> = vec![];

fn add_to_log_store(message: String) {
    unsafe {
        LOG_STORE.push(message);

        if LOG_STORE.len() > MAX_LOGS.into() {
            flush_logs();
        }
    }
}

// TODO use custom log rotation strategy
fn flush_logs() {
    let project_settings = ProjectSettings::singleton();

    let path = project_settings.globalize_path(GodotString::from("user://vpuppr.log"));

    let mut opts = std::fs::OpenOptions::new();
    opts.truncate(false).write(true).create(true);

    unsafe {
        match opts.open(path.to_string()) {
            Ok(mut file) => {
                for log in LOG_STORE.iter() {
                    if let Err(e) = file.write_all(log.as_bytes()) {
                        godot_error!("{e}");
                        break;
                    }
                }
            }
            Err(e) => godot_error!("{e}"),
        };

        LOG_STORE.clear();
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LogLevel {
    Info,
    Warn,
    Error,

    Debug,
}

#[derive(Debug, GodotClass)]
struct Logger {
    name: GodotString,
}

#[godot_api]
impl RefCountedVirtual for Logger {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self::new(GodotString::from("DefaultLogger"))
    }
}

#[godot_api]
impl Logger {
    #[func]
    fn create(name: GodotString) -> Gd<Logger> {
        Gd::new(Self::new(name))
    }

    #[func]
    fn info(&self, message: Variant) {
        self.log(LogLevel::Info, &mut message.stringify().to_string());
    }

    #[func]
    fn warn(&self, message: Variant) {
        self.log(LogLevel::Warn, &mut message.stringify().to_string());
    }

    #[func]
    fn error(&self, message: Variant) {
        self.log(LogLevel::Error, &mut message.stringify().to_string());
    }

    #[func]
    fn debug(&self, message: Variant) {
        self.log(LogLevel::Debug, &mut message.stringify().to_string());
    }
}

impl Logger {
    fn new(name: GodotString) -> Self {
        Self { name }
    }

    fn log(&self, level: LogLevel, message: &mut String) {
        let message = self.insert_metadata(level, message);

        godot_print!("{message}");
        add_to_log_store(message);
    }

    fn insert_metadata(&self, level: LogLevel, message: &String) -> String {
        let datetime = chrono::Local::now();
        let date = datetime.date_naive();
        let time = datetime.time();
        let time = format!("{}_{}", date.format("%Y-%m-%d"), time.format("%H:%M:%S"));

        format!("[{:?}] {} {} {}", level, time, self.name, message)
    }
}
