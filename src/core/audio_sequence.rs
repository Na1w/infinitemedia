//! Audio sequence representation for the timeline.
//!
//! This module defines audio sequences that can be added to the timeline.
//! The audio engine reads these sequences and generates sound procedurally.

/// An audio sequence that generates sound between a start and end time.
///
/// Instead of individual triggers, an AudioSequence defines a procedural
/// pattern that plays over a time range.
#[derive(Clone, Debug)]
pub struct AudioSequence {
    /// Start time in seconds.
    pub start: f32,
    /// End time in seconds.
    pub end: f32,
    /// Beats per minute.
    pub bpm: f32,
    /// Steps per beat (e.g., 4 for 16th notes at quarter note beats).
    pub steps_per_beat: u8,
    /// Base pattern as multipliers from root frequency.
    pub base_pattern: Vec<f32>,
    /// Accent pattern (1.0 = accented, 0.0 = normal).
    pub accents: Vec<f32>,
    /// Transposition values for each pattern cycle.
    pub transpositions: Vec<f32>,
    /// Root frequency in Hz.
    pub root_freq: f32,
    /// Duration of fade-in in seconds.
    pub fade_in: f32,
    /// Duration of fade-out in seconds.
    pub fade_out: f32,
}

impl AudioSequence {
    /// Creates a new audio sequence with default values.
    pub fn new(start: f32, end: f32, bpm: f32) -> Self {
        Self {
            start,
            end,
            bpm,
            steps_per_beat: 4, // 16th notes
            base_pattern: vec![1.0; 16],
            accents: vec![0.0; 16],
            transpositions: vec![1.0],
            root_freq: 55.0,
            fade_in: 0.0,
            fade_out: 0.0,
        }
    }

    /// Set the fade-in and fade-out durations in seconds.
    pub fn with_fades(mut self, fade_in: f32, fade_out: f32) -> Self {
        self.fade_in = fade_in;
        self.fade_out = fade_out;
        self
    }

    /// Set the base frequency pattern.
    pub fn with_pattern(mut self, pattern: Vec<f32>) -> Self {
        self.base_pattern = pattern;
        self
    }

    /// Set accent pattern (1.0 = accented).
    pub fn with_accents(mut self, accents: Vec<f32>) -> Self {
        self.accents = accents;
        self
    }

    /// Set transpositions for pattern cycles.
    pub fn with_transpositions(mut self, trans: Vec<f32>) -> Self {
        self.transpositions = trans;
        self
    }

    /// Set root frequency in Hz.
    pub fn with_root(mut self, freq: f32) -> Self {
        self.root_freq = freq;
        self
    }

    /// Set steps per beat (e.g., 4 for 16th notes).
    pub fn with_steps_per_beat(mut self, steps: u8) -> Self {
        self.steps_per_beat = steps;
        self
    }

    /// Returns the duration of one step in seconds.
    pub fn step_duration(&self) -> f32 {
        60.0 / (self.bpm * self.steps_per_beat as f32)
    }

    /// Returns the number of steps in the pattern.
    pub fn num_steps(&self) -> usize {
        self.base_pattern.len()
    }
}
