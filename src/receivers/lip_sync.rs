/*!
A receiver for lip syncing data.

This implementation takes audio input from Godot, converts that audio to text,
and then generates phonemes from them. The text and phonemes are both usable
from vpuppr, allowing for both model lip-sync and actions based off of voice commands.
*/

use godot::prelude::*;

// TODO maybe use https://github.com/tazz4843/whisper-rs instead of own impl
// use in conjunction with https://github.com/Dalvany/rphonetic

struct LipSync {}
