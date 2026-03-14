//! > ⚠️ **Work in Progress (WIP)** ⚠️
//! > This library is currently in early, experimental development. The API is highly unstable and will likely undergo significant breaking changes.

pub mod core;
pub mod engine;

pub use core::audio_sequence::AudioSequence;
pub use core::state::MediaState;
pub use core::timeline::{Interval, Timeline};
pub use engine::{MediaAction, MediaEngine, SequenceContent, TransitionKind};
