use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicUsize};

/// Shared state for media management and synchronization between audio and graphics.
///
/// Most values are stored as `AtomicU32` containing bit-casted `f32` values to allow
/// lock-free updates across threads.
pub struct MediaState {
    /// Delay applied to visual events relative to audio to account for buffering.
    pub visual_delay: f32,
    /// The current audio sample rate (stored as bits of f32).
    pub sample_rate: Arc<AtomicU32>,
    /// Current audio playback time in seconds (stored as bits of f32).
    pub audio_time: Arc<AtomicU32>,
    /// Generic parameters for application-specific synchronization (e.g. envelopes).
    pub parameters: Vec<Arc<AtomicU32>>,
    /// Buffer containing the most recent audio samples for waveform visualization.
    pub waveform: Arc<Vec<AtomicU32>>,
    /// Current write position in the cyclic waveform buffer.
    pub waveform_ptr: Arc<AtomicUsize>,
}

impl MediaState {
    /// Creates a new MediaState with the given number of generic parameters.
    pub fn new(num_params: usize) -> Self {
        Self {
            visual_delay: 0.0,
            sample_rate: Arc::new(AtomicU32::new(44100.0f32.to_bits())),
            audio_time: Arc::new(AtomicU32::new(0f32.to_bits())),
            parameters: (0..num_params)
                .map(|_| Arc::new(AtomicU32::new(0)))
                .collect(),
            waveform: Arc::new((0..512).map(|_| AtomicU32::new(0)).collect()),
            waveform_ptr: Arc::new(AtomicUsize::new(0)),
        }
    }
}
