use godot::prelude::*;

static mut LOG_STORE: Vec<String> = vec![];

fn add_to_log_store(message: String) {
    unsafe {
        LOG_STORE.push(message);
    }
}

#[derive(Debug, GodotClass)]
struct Logger {}

#[godot_api]
impl RefCountedVirtual for Logger {
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        Self {}
    }
}

#[godot_api]
impl Logger {
    #[func]
    fn create() -> Gd<Logger> {
        Gd::new(Self {})
    }

    #[func]
    fn info(&self, message: Variant) {
        let message = message.stringify().to_string();

        godot_print!("{message}");
        add_to_log_store(message);
    }
}
