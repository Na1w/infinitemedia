use crate::core::MediaState;
use crate::core::Timeline;
use crate::core::audio_sequence::AudioSequence;
use infinitegfx_core::GfxChain;
use infinitegfx_core::StandardGlobals;
use infinitegfx_core::core::{GfxFrame, GfxFrameProcessor, GfxHandle, RenderContext};
use infinitegfx_core::effects::ShaderNode;
use std::sync::Arc;

/// Types of transitions supported by the engine.
#[derive(Clone, Copy, Debug)]
pub enum TransitionKind {
    /// A 3D flip effect between two scenes.
    Flip,
    /// A space-bending distortion transition.
    SpaceBend,
    /// A simple linear crossfade between two scenes.
    Crossfade,
}

/// Actions that can be performed in the timeline.
#[derive(Clone, Debug)]
pub enum MediaAction {
    /// Renders a specific scene by its index in the content.
    Scene(usize),
    /// Performs a transition between two scenes.
    Transition {
        /// Index of the source scene.
        from: usize,
        /// Index of the target scene.
        to: usize,
        /// The type of transition to apply.
        kind: TransitionKind,
    },
    /// Sets a shared state parameter (id, value).
    Parameter(usize, f32),
    /// Triggers a momentary event (e.g. kick, voice phoneme).
    Trigger(usize),
    /// An audio sequence that plays over a time range.
    AudioSequence(AudioSequence),
}

/// Container for the sequence content including timeline and scenes.
#[derive(Default)]
pub struct SequenceContent {
    /// The timeline defining when each action occurs.
    pub timeline: Arc<Timeline<MediaAction>>,
    /// The list of graphics chains (scenes) available for rendering.
    pub scenes: Vec<GfxChain>,
}

impl SequenceContent {
    /// Creates a new, empty sequence content.
    pub fn new() -> Self {
        Self {
            timeline: Arc::new(Timeline::new()),
            scenes: Vec::new(),
        }
    }
}

/// Type alias for a function that builds StandardGlobals.
pub type GlobalsBuilder = fn(&MediaState, f32, f32, u32, u32, &mut f32) -> StandardGlobals;

/// The core engine responsible for executing the timeline and rendering scenes.
pub struct MediaEngine {
    /// Shared state containing audio-reactive parameters.
    pub state: Arc<MediaState>,
    /// The content (scenes and timeline) to be rendered.
    pub content: SequenceContent,
    /// Internal shader node for the Flip transition.
    pub flip: Option<ShaderNode>,
    /// Internal shader node for the Warp effect (if used).
    pub warp: Option<ShaderNode>,
    /// Internal shader node for the SpaceBend transition.
    pub space_bend: Option<ShaderNode>,
    /// Internal shader node for the Curtain effect (if used).
    pub curtain: Option<ShaderNode>,
    /// Internal shader node for the Crossfade transition.
    pub crossfade: Option<ShaderNode>,
    /// GPU buffer for global shader uniforms.
    pub globals_buf: Option<wgpu::Buffer>,
    /// Bind group for global shader uniforms.
    pub bind_group: Option<wgpu::BindGroup>,
    /// Optional custom builder for global shader uniforms.
    pub globals_builder: Option<GlobalsBuilder>,
    /// Custom state value for the globals builder.
    pub user_data: f32,
    /// The timestamp of the last rendered frame.
    pub last_time: f32,
}

impl MediaEngine {
    /// Creates a new `MediaEngine` instance with the given shared state.
    pub fn new(state: Arc<MediaState>) -> Self {
        Self {
            state,
            content: SequenceContent::new(),
            flip: None,
            warp: None,
            space_bend: None,
            curtain: None,
            crossfade: None,
            globals_buf: None,
            bind_group: None,
            globals_builder: None,
            user_data: 0.0,
            last_time: 0.0,
        }
    }

    /// Sets a custom builder for global shader uniforms.
    pub fn with_globals_builder(mut self, builder: GlobalsBuilder) -> Self {
        self.globals_builder = Some(builder);
        self
    }

    /// Replaces the current timeline with a shared one.
    pub fn with_timeline(mut self, timeline: Arc<Timeline<MediaAction>>) -> Self {
        self.content.timeline = timeline;
        self
    }

    pub fn with_scene(mut self, start: f32, end: f32, chain: GfxChain) -> Self {
        let idx = self.content.scenes.len();
        self.content.scenes.push(chain);
        Arc::make_mut(&mut self.content.timeline).add(start, end, MediaAction::Scene(idx));
        self
    }

    pub fn with_existing_scene(mut self, start: f32, end: f32, scene_idx: usize) -> Self {
        Arc::make_mut(&mut self.content.timeline).add(start, end, MediaAction::Scene(scene_idx));
        self
    }

    pub fn with_transition(
        mut self,
        start: f32,
        end: f32,
        from: usize,
        to: usize,
        kind: TransitionKind,
    ) -> Self {
        Arc::make_mut(&mut self.content.timeline).add(
            start,
            end,
            MediaAction::Transition { from, to, kind },
        );
        self
    }

    pub fn with_parameter(mut self, start: f32, end: f32, id: usize, value: f32) -> Self {
        Arc::make_mut(&mut self.content.timeline).add(
            start,
            end,
            MediaAction::Parameter(id, value),
        );
        self
    }

    pub fn with_trigger(mut self, start: f32, end: f32, id: usize) -> Self {
        Arc::make_mut(&mut self.content.timeline).add(start, end, MediaAction::Trigger(id));
        self
    }

    /// High-level method to render a frame directly to a texture view.
    /// Handles encoder creation and submission internally.
    pub fn render_to_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        time: f32,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("MediaEngineEncoder"),
        });

        {
            let mut frame = GfxFrame {
                device,
                queue,
                encoder: &mut encoder,
                target_view,
                width,
                height,
                time,
            };

            self.render_frame(&mut frame);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Records rendering commands for the current frame into the provided encoder.
    pub fn render_frame(&mut self, frame: &mut GfxFrame) {
        use std::sync::atomic::Ordering;

        let time = frame.time;
        let dt = if self.last_time > 0.0 {
            (time - self.last_time).clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.last_time = time;

        let active_events = self.content.timeline.active_at(time);

        // 1. Process Parameter and Trigger events
        for event in active_events.clone() {
            match event.data {
                MediaAction::Parameter(id, val) => {
                    if id < self.state.parameters.len() {
                        self.state.parameters[id].store(val.to_bits(), Ordering::Relaxed);
                    }
                }
                MediaAction::Trigger(_id) => {}
                _ => {}
            }
        }

        let builder = self
            .globals_builder
            .unwrap_or(|_state, t, _dt, w, h, _ud| StandardGlobals {
                time: t,
                kick: 0.0,
                sweep: 0.0,
                res_x: w as f32,
                res_y: h as f32,
                fade: 1.0,
                p2: 0.0,
                p3: 0.0,
            });

        let globals = builder(
            &self.state,
            time,
            dt,
            frame.width,
            frame.height,
            &mut self.user_data,
        );

        if let Some(buf) = &self.globals_buf {
            frame
                .queue
                .write_buffer(buf, 0, bytemuck::bytes_of(&globals));
        }

        let bind_group = self
            .bind_group
            .as_ref()
            .expect("MediaEngine not initialized");
        let globals_buf = self.globals_buf.as_ref().unwrap();

        // 2. Find the primary visual action
        let visual_event = active_events.iter().find(|e| {
            matches!(
                e.data,
                MediaAction::Scene(_) | MediaAction::Transition { .. }
            )
        });

        if let Some(segment) = visual_event {
            match segment.data {
                MediaAction::Scene(idx) => {
                    let scene = &mut self.content.scenes[idx];
                    scene.update(time, frame.queue, globals_buf);
                    scene.render(RenderContext {
                        device: frame.device,
                        encoder: frame.encoder,
                        target_view: frame.target_view,
                        input_view: None,
                        globals_bind_group: bind_group,
                        time,
                        queue: frame.queue,
                        globals_buf,
                    });
                }
                MediaAction::Transition { from, to, kind } => {
                    let progress = segment.progress(time);

                    self.content.scenes[from].update(time, frame.queue, globals_buf);
                    self.content.scenes[to].update(time, frame.queue, globals_buf);

                    self.content.scenes[from].render_to_self(RenderContext {
                        device: frame.device,
                        encoder: frame.encoder,
                        target_view: frame.target_view,
                        input_view: None,
                        globals_bind_group: bind_group,
                        time,
                        queue: frame.queue,
                        globals_buf,
                    });
                    self.content.scenes[to].render_to_self(RenderContext {
                        device: frame.device,
                        encoder: frame.encoder,
                        target_view: frame.target_view,
                        input_view: None,
                        globals_bind_group: bind_group,
                        time,
                        queue: frame.queue,
                        globals_buf,
                    });

                    let v_from = self.content.scenes[from].last_result_view().unwrap();
                    let v_to = self.content.scenes[to].last_result_view().unwrap();

                    let ctx = RenderContext {
                        device: frame.device,
                        encoder: frame.encoder,
                        target_view: frame.target_view,
                        input_view: None,
                        globals_bind_group: bind_group,
                        time,
                        queue: frame.queue,
                        globals_buf,
                    };

                    match kind {
                        TransitionKind::Flip => {
                            if let Some(f) = &mut self.flip {
                                let uniforms = [
                                    (progress * std::f32::consts::PI * 2.0).sin() * 0.5,
                                    progress * std::f32::consts::PI,
                                    progress * std::f32::consts::PI * 2.0,
                                    2.0 + (progress * std::f32::consts::PI).sin() * 3.0,
                                    0.0,
                                    0.0,
                                    0.0,
                                    0.0,
                                ];
                                if let Some(buf) = f.uniform_buf.as_ref() {
                                    frame.queue.write_buffer(
                                        buf,
                                        0,
                                        bytemuck::cast_slice(&uniforms),
                                    );
                                }
                                f.render_two(ctx, v_to, v_from);
                            }
                        }
                        TransitionKind::SpaceBend => {
                            if let Some(sb) = &mut self.space_bend {
                                if let Some(buf) = sb.uniform_buf.as_ref() {
                                    frame.queue.write_buffer(
                                        buf,
                                        0,
                                        bytemuck::cast_slice(&[progress, 0.0, 0.0, 0.0]),
                                    );
                                }
                                sb.render_two(ctx, v_from, v_to);
                            }
                        }
                        TransitionKind::Crossfade => {
                            if let Some(cf) = &mut self.crossfade {
                                if let Some(buf) = cf.uniform_buf.as_ref() {
                                    frame.queue.write_buffer(
                                        buf,
                                        0,
                                        bytemuck::cast_slice(&[progress, 0.0, 0.0, 0.0]),
                                    );
                                }
                                cf.render_two(ctx, v_from, v_to);
                            }
                        }
                    }
                }
                _ => {}
            }
        } else if let Some(last) = self.content.scenes.last_mut() {
            last.update(time, frame.queue, globals_buf);
            last.render(RenderContext {
                device: frame.device,
                encoder: frame.encoder,
                target_view: frame.target_view,
                input_view: None,
                globals_bind_group: bind_group,
                time,
                queue: frame.queue,
                globals_buf,
            });
        }
    }
}

impl GfxFrameProcessor for MediaEngine {
    fn init(&mut self, handle: &GfxHandle, _globals_layout: &wgpu::BindGroupLayout) {
        let (globals_buf, internal_globals_layout, bind_group) =
            handle.create_globals_buffer(std::mem::size_of::<StandardGlobals>() as u64);

        use infinitegfx_core::effects as fx;

        let mut flip = fx::flip();
        flip.init(handle, &internal_globals_layout);
        let mut warp = fx::warp();
        warp.init(handle, &internal_globals_layout);
        let mut space_bend = fx::space_bend();
        space_bend.init(handle, &internal_globals_layout);
        let mut curtain = fx::curtain();
        curtain.init(handle, &internal_globals_layout);
        let mut crossfade = fx::crossfade();
        crossfade.init(handle, &internal_globals_layout);

        for scene in &mut self.content.scenes {
            scene.init(handle, &internal_globals_layout);
        }

        self.globals_buf = Some(globals_buf);
        self.bind_group = Some(bind_group);
        self.flip = Some(flip);
        self.warp = Some(warp);
        self.space_bend = Some(space_bend);
        self.curtain = Some(curtain);
        self.crossfade = Some(crossfade);
    }

    fn update(&mut self, time: f32, queue: &wgpu::Queue, globals_buf: &wgpu::Buffer) {
        let active_events = self.content.timeline.active_at(time);
        for event in active_events {
            match event.data {
                MediaAction::Scene(idx) => {
                    self.content.scenes[idx].update(time, queue, globals_buf);
                }
                MediaAction::Transition { from, to, .. } => {
                    self.content.scenes[from].update(time, queue, globals_buf);
                    self.content.scenes[to].update(time, queue, globals_buf);
                }
                _ => {}
            }
        }
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        for scene in &mut self.content.scenes {
            scene.resize(device, w, h);
        }
    }

    fn render<'a>(&'a mut self, ctx: RenderContext<'a>) {
        let mut frame = GfxFrame {
            device: ctx.device,
            queue: ctx.queue,
            encoder: ctx.encoder,
            target_view: ctx.target_view,
            width: 0,
            height: 0,
            time: ctx.time,
        };
        self.render_frame(&mut frame);
    }
}
