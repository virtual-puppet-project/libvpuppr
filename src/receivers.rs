pub(crate) mod lip_sync;
pub(crate) mod meow_face;

use godot::{engine::global::Error, prelude::*};

use crate::puppets::{puppet_2d::Puppet2d, puppet_3d::Puppet3d};

/// A tracking data receiver.
trait Receiver<T: GodotClass> {
    /// Create an instance of the receiver.
    fn create_inner(data: &Dictionary) -> Option<Gd<T>>;

    /// Start the receiver.
    ///
    /// # Return
    /// The PID if starting was successful or -1 on failure.
    fn start_inner(&mut self) -> Error;

    /// Stop the receiver.
    ///
    /// # Return
    /// OK on success or an error code.
    fn stop_inner(&mut self) -> Error;

    /// Check for and apply data.
    fn poll_inner(&mut self);

    /// Applies data to a Puppet3d.
    fn handle_puppet3d_inner(&self, puppet: Gd<Puppet3d>);

    /// Applies data to a Puppet2d.
    fn handle_puppet2d_inner(&self, puppet: Gd<Puppet2d>);
}

/// Automatically bind these receiver methods to Godot.
macro_rules! bind_receiver_to_godot {
    ($name:ident) => {
        #[godot_api]
        impl $name {
            #[func]
            fn create(data: Dictionary) -> Option<Gd<$name>> {
                Self::create_inner(&data)
            }

            #[func]
            fn start(&mut self) -> Error {
                Self::start_inner(self)
            }

            #[func]
            fn stop(&mut self) -> Error {
                Self::stop_inner(self)
            }

            #[func]
            fn poll(&mut self) {
                Self::poll_inner(self);
            }

            #[func]
            fn handle_puppet3d(&self, puppet: Gd<Puppet3d>) {
                Self::handle_puppet3d_inner(self, puppet);
            }

            #[func]
            fn visit_puppet2d(&self, puppet: Gd<Puppet2d>) {
                Self::handle_puppet2d_inner(self, puppet);
            }
        }
    };
}
pub(crate) use bind_receiver_to_godot;
