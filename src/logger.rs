use std::io::Write;

use godot::{engine::ProjectSettings, prelude::*};

const MAX_LOGS: u8 = 100;
static mut LOG_STORE: Vec<String> = vec![];

/// Add a `message` to the static `LOG_STORE`.
///
/// # Safety
/// Global access is needed since a Godot autoload might not be available for writing
/// when the first logger is initialized.
fn add_to_log_store(message: String) {
    unsafe {
        LOG_STORE.push(message);

        if LOG_STORE.len() > MAX_LOGS.into() {
            flush_logs();
        }
    }
}

// TODO use custom log rotation strategy
/// Flush all logs from the static `LOG_STORE` into a file.
///
/// # Safety
/// Global access is needed for the log store since a Godot autoload might not be available for
/// writing when the first logger is initialized.
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

/// The level to log outputs at.
#[derive(Debug, PartialEq, Eq)]
enum LogLevel {
    Info,
    Warn,
    Error,

    Debug,
    Global,
}

/// A structured logger that helps work around Godot dropping logs when it crashes.
#[derive(Debug, GodotClass)]
pub struct Logger {
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
    /// Create a new `Logger` in Godot with the given name. Loggers may have
    /// duplicate names but this is **_strongly_** discouraged.
    #[func]
    pub fn create(name: GodotString) -> Gd<Logger> {
        Gd::new(Self::new(name))
    }

    pub fn set_name(&mut self, name: GodotString) {
        self.name = name.into();
    }

    /// Send a log at the `Info` log level. Logs are printed to stdout.
    #[func]
    pub fn info(&self, message: Variant) {
        self.log(LogLevel::Info, &mut message.stringify().to_string());
    }

    /// Send a log at the `Warn` log level. Logs are printed to stdout.
    #[func]
    pub fn warn(&self, message: Variant) {
        self.log(LogLevel::Warn, &mut message.stringify().to_string());
    }

    /// Send a log at the `Error` log level. Logs are printed to stderr.
    #[func]
    pub fn error(&self, message: Variant) {
        self.log(LogLevel::Error, &mut message.stringify().to_string());
    }

    /// Send a log at the `Debug` log leve. Logs are printed to stdout.
    #[func]
    pub fn debug(&self, message: Variant) {
        #[cfg(debug_assertions)]
        self.log(LogLevel::Debug, &mut message.stringify().to_string());
    }

    /// Send a log using an anonymous logger. Logs are printed to stdout.
    #[func]
    pub fn global(source: GodotString, message: Variant) {
        let message = insert_metadata(
            source.to_string(),
            &LogLevel::Global,
            &mut message.stringify().to_string(),
        );

        godot_print!("{message}");
        add_to_log_store(message);
    }
}

impl Logger {
    /// Create a new logger with the given name.
    fn new(name: GodotString) -> Self {
        Self { name }
    }

    /// Use the given `level` and `message` to send a log and add the log to
    /// the static `LOG_STORE`.
    fn log(&self, level: LogLevel, message: &mut String) {
        let message = insert_metadata(self.name.to_string(), &level, message);

        if level != LogLevel::Error {
            godot_print!("{message}");
        } else {
            godot_error!("{message}");
        }
        add_to_log_store(message);
    }
}

/// Modify a given log message with the logger name, log level, and datetime.
fn insert_metadata(logger_name: String, level: &LogLevel, message: &String) -> String {
    let datetime = chrono::Local::now();
    let date = datetime.date_naive();
    let time = datetime.time();
    let time = format!("{}_{}", date.format("%Y-%m-%d"), time.format("%H:%M:%S"));

    format!("[{:?}] {} {} {}", level, time, logger_name, message)
}
