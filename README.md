# Vpuppr Rust Lib

A [GDExtension](https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/what_is_gdextension.html) made
with [godot-rust](https://github.com/godot-rust/gdext) for [vpuppr](https://github.com/virtual-puppet-project/vpuppr).

This repository is meant to be used as a git submodule in the main vpuppr repository.

## Building

Run `python build.py --help` for possible options. This is a simple wrapper around
`cargo build` and `cargo build --release` that also renames the output libraries
to `libvpuppr.dll` or `libvpuppr.so` depending on platform.

## License

MPL-2.0
