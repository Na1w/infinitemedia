/// Represents a time interval with associated data.
#[derive(Clone, Debug)]
pub struct Interval<T> {
    pub start: f32,
    pub end: f32,
    pub data: T,
}

impl<T> Interval<T> {
    /// Returns the time relative to the start of the interval.
    #[allow(dead_code)]
    pub fn local_time(&self, absolute_time: f32) -> f32 {
        absolute_time - self.start
    }

    /// Returns the progress of the interval (0.0 to 1.0) based on absolute time.
    pub fn progress(&self, absolute_time: f32) -> f32 {
        if self.end <= self.start {
            return 1.0;
        }
        ((absolute_time - self.start) / (self.end - self.start)).clamp(0.0, 1.0)
    }

    /// Returns true if the interval contains the given time.
    pub fn contains(&self, time: f32) -> bool {
        time >= self.start && time < self.end
    }
}

/// A timeline consisting of multiple sequential or overlapping intervals.
#[derive(Clone, Debug)]
pub struct Timeline<T> {
    entries: Vec<Interval<T>>,
}

impl<T: Clone> Default for Timeline<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Timeline<T> {
    /// Creates a new, empty timeline.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds a new interval to the timeline.
    pub fn add(&mut self, start: f32, end: f32, data: T) {
        self.entries.push(Interval { start, end, data });
    }

    /// Returns all intervals active at the given time.
    #[allow(dead_code)]
    pub fn active_at(&self, time: f32) -> Vec<&Interval<T>> {
        self.entries.iter().filter(|i| i.contains(time)).collect()
    }

    /// Finds the first interval active at the given time.
    pub fn find_active(&self, time: f32) -> Option<&Interval<T>> {
        self.entries.iter().find(|i| i.contains(time))
    }

    /// Finds all intervals that start within the given time range [start, end).
    pub fn find_in_range(&self, start: f32, end: f32) -> Vec<&Interval<T>> {
        self.entries
            .iter()
            .filter(|i| i.start >= start && i.start < end)
            .collect()
    }

    /// Returns the end time of the last interval in the timeline.
    pub fn duration(&self) -> f32 {
        self.entries.iter().map(|i| i.end).fold(0.0, f32::max)
    }
}
