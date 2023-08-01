mod meow_face;

use godot::prelude::*;

trait Receiver<T: GodotClass> {
    fn create_inner(data: Dictionary) -> Gd<T>;

    fn start_inner(data: Dictionary) -> i64;

    fn stop_inner() -> u32;
}

/// Automatically bind these receiver methods to Godot.
macro_rules! bind_receiver_to_godot {
    ($name:ident) => {
        #[godot_api]
        impl $name {
            #[func]
            fn create(data: Dictionary) -> Gd<$name> {
                Self::create_inner(data)
            }

            #[func]
            fn start(data: Dictionary) -> i64 {
                Self::start_inner(data)
            }

            #[func]
            fn stop() -> u32 {
                Self::stop_inner()
            }
        }
    };
}
pub(crate) use bind_receiver_to_godot;
