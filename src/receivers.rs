mod lip_sync;
mod meow_face;

use godot::{engine::global::Error, prelude::*};

/// A tracking data receiver.
trait Receiver<T: GodotClass> {
    /// Create an instance of the receiver.
    fn create_inner(data: Dictionary) -> Gd<T>;

    /// Start the receiver.
    ///
    /// # Return
    /// The PID if starting was successful or -1 on failure.
    fn start_inner(data: Dictionary) -> i64;

    /// Stop the receiver.
    ///
    /// # Return
    /// OK on success or an error code.
    fn stop_inner() -> Error;
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
            fn stop() -> Error {
                Self::stop_inner()
            }
        }
    };
}
pub(crate) use bind_receiver_to_godot;
