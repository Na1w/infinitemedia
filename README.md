# infinitemedia-core

> ⚠️ **Work in Progress (WIP)** ⚠️
> This library is currently in early, experimental development. The API is highly unstable and will likely undergo significant breaking changes. Use in production environments is not recommended at this stage.

A generic timeline and sequence engine for orchestrating and synchronizing audio and visual events in Rust.

**infinitemedia-core** is part of the `infinite*` ecosystem (alongside `infinitedsp-core` and `infinitegfx-core`). It provides a high-level state machine and timeline structure, making it easy to create complex, reactive multimedia applications, demos, and games.

It natively integrates with `infinitegfx-core`'s shader and rendering pipeline via the `MediaEngine`, allowing you to seamlessly script scene changes, transitions, parameter automations, and trigger audio events on a shared, frame-accurate timeline.

## Features

- **Timeline Management**: Schedule events, scenes, and transitions using a precise floating-point timeline.
- **Media Engine**: A high-level orchestration container that manages rendering contexts and states.
- **Audio Sequences**: Define procedural music and sound patterns that translate to timeline triggers and dynamic fade parameters.
- **State Synchronization**: Lock-free, thread-safe generic `MediaState` struct specifically designed to share variables (like audio waveform buffers or LFO envelopes) between audio processing threads and the GPU render loop.
- **Transitions**: Built-in 3D flip, space-bend, and crossfade transition logic.
- **Agnostic Architecture**: Designed to be extended. `MediaEngine` accepts custom builders for global uniform buffers, so your GPU logic stays domain-specific while the engine handles the scheduling.

## Getting Started

Add `infinitemedia-core` to your `Cargo.toml`:

```toml
[dependencies]
infinitemedia-core = "0.1.0"
```

## Basic Usage

Here is a simplified example of how you can build a timeline using the engine:

```rust
use infinitemedia_core::{MediaEngine, MediaState, Timeline, TransitionKind, MediaAction};
use infinitegfx_core::core::GfxChain;
use std::sync::Arc;

// 1. Initialize shared state
let state = Arc::new(MediaState::new(8)); 
let timeline = Arc::new(Timeline::new());

// 2. Build the Media Engine
let mut engine = MediaEngine::new(state.clone())
    .with_timeline(timeline);

// 3. Define scenes and transitions on the timeline
// (Assuming `scene_one` and `scene_two` are functions returning a `GfxChain`)
engine = engine
    .with_scene(0.0, 10.0, scene_one()) // Scene 1 plays from 0s to 10s
    .with_scene(10.0, 20.0, scene_two()) // Scene 2 plays from 10s to 20s
    .with_transition(10.0, 12.0, 0, 1, TransitionKind::Crossfade); // Crossfade over 2 seconds

// 4. In your render loop, you just pass the time:
// engine.render_to_view(&device, &queue, &view, width, height, current_time);
```

## The Ecosystem

`infinitemedia-core` works best when paired with its sister libraries:
- [**infinitedsp-core**](https://crates.io/crates/infinitedsp-core) - For generating the audio and procedural sound effects you schedule on the timeline.
- [**infinitegfx-core**](https://crates.io/crates/infinitegfx-core) - For defining the visual shader nodes, scenes, and post-processing chains that the media engine renders.

## License

Licensed under the MIT license.
