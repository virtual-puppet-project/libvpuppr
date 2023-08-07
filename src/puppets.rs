pub mod puppet_2d;
pub mod puppet_3d;

/// How to handle the various tracking data sources.
///
/// NOTE: this isn't _really_ a visitor since only the data is being passed along,
/// but it's close enough.
///
/// IMPORTANT: Since mediapipe is implemented in a separate GDExtension, we cannot access
/// any types from there. However, we can map that data to a `Dictionary` and visit that
/// instead.
pub(crate) trait Visitor {
    fn visit_mediapipe_inner(&mut self, _data: godot::prelude::Dictionary) {}

    fn visit_meow_face_inner(&mut self, _data: &crate::receivers::meow_face::Data) {}
}

// TODO unused until #[godot_api] supports multiple impls
macro_rules! bind_visitor_to_godot {
    () => {};
}
pub(crate) use bind_visitor_to_godot;
