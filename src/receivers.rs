pub(crate) mod lip_sync;
// pub(crate) mod meow_face;

use godot::{engine::global::Error, prelude::*};

use crate::puppets::{puppet_2d::Puppet2d, puppet_3d::Puppet3d};

/// A tracking data receiver.
trait Receiver<T: GodotClass> {
    /// Create an instance of the receiver.
    fn create(data: &Dictionary) -> Option<Gd<T>>;

    /// Start the receiver.
    ///
    /// # Return
    /// The PID if starting was successful or -1 on failure.
    fn start(&mut self) -> Error;

    /// Stop the receiver.
    ///
    /// # Return
    /// OK on success or an error code.
    fn stop(&mut self) -> Error;

    /// Check for and apply data.
    fn poll(&mut self);

    /// Applies data to a Puppet3d.
    fn handle_puppet3d(&self, puppet: Gd<Puppet3d>);

    /// Applies data to a Puppet2d.
    fn handle_puppet2d(&self, puppet: Gd<Puppet2d>);
}

/// Automatically bind these receiver methods to Godot.
macro_rules! bind_receiver_to_godot {
    ($name:ident) => {
        #[godot_api]
        impl $name {
            #[func(rename = create)]
            fn create_bound(data: Dictionary) -> Option<Gd<$name>> {
                Self::create(&data)
            }

            #[func(rename = start)]
            fn start_bound(&mut self) -> Error {
                Self::start(self)
            }

            #[func(rename = stop)]
            fn stop_bound(&mut self) -> Error {
                Self::stop(self)
            }

            #[func(rename = poll)]
            fn poll_bound(&mut self) {
                Self::poll(self);
            }

            #[func(rename = handle_puppet3d)]
            fn handle_puppet3d_bound(&self, puppet: Gd<Puppet3d>) {
                Self::handle_puppet3d(self, puppet);
            }

            #[func(rename = handle_puppet2d)]
            fn visit_puppet2d_bound(&self, puppet: Gd<Puppet2d>) {
                Self::handle_puppet2d(self, puppet);
            }
        }
    };
}
pub(crate) use bind_receiver_to_godot;
