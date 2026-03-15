use intervaltree::IntervalTree;
use std::cmp::Ordering;

#[derive(PartialEq, Clone, Copy, Debug)]
struct IntervalFloat(f32);

impl Eq for IntervalFloat {}

impl PartialOrd for IntervalFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntervalFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

/// Represents a time interval with associated data.
#[derive(Clone, Debug)]
pub struct Interval<T> {
    pub start: f32,
    pub end: f32,
    pub data: T,
}

impl<T> Interval<T> {
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
    tree: Option<IntervalTree<IntervalFloat, T>>,
    pending: Vec<Interval<T>>,
    duration: f32,
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
            tree: None,
            pending: Vec::new(),
            duration: 0.0,
        }
    }

    /// Adds a new interval to the timeline.
    pub fn add(&mut self, start: f32, end: f32, data: T) {
        self.duration = self.duration.max(end);
        self.pending.push(Interval { start, end, data });
        self.tree = None;
    }

    fn ensure_tree(&mut self) {
        if self.tree.is_none() && !self.pending.is_empty() {
            let elements: Vec<_> = self
                .pending
                .iter()
                .map(|i| (IntervalFloat(i.start)..IntervalFloat(i.end), i.data.clone()))
                .collect();
            self.tree = Some(IntervalTree::from_iter(elements));
        }
    }

    /// Returns all intervals active at the given time.
    pub fn active_at(&self, time: f32) -> Vec<Interval<T>> {
        if let Some(tree) = &self.tree {
            let t = IntervalFloat(time);
            tree.query(t..IntervalFloat(time + 0.00001))
                .map(|element| Interval {
                    start: element.range.start.0,
                    end: element.range.end.0,
                    data: element.value.clone(),
                })
                .collect()
        } else {
            self.pending
                .iter()
                .filter(|i| i.contains(time))
                .cloned()
                .collect()
        }
    }

    /// Finds the first interval active at the given time.
    pub fn find_active(&self, time: f32) -> Option<Interval<T>> {
        if let Some(tree) = &self.tree {
            let t = IntervalFloat(time);
            tree.query(t..IntervalFloat(time + 0.00001))
                .next()
                .map(|element| Interval {
                    start: element.range.start.0,
                    end: element.range.end.0,
                    data: element.value.clone(),
                })
        } else {
            self.pending.iter().find(|i| i.contains(time)).cloned()
        }
    }

    /// Finds all intervals that start within the given time range [start, end).
    pub fn find_in_range(&self, start: f32, end: f32) -> Vec<Interval<T>> {
        if let Some(tree) = &self.tree {
            let s = IntervalFloat(start);
            let e = IntervalFloat(end);
            tree.query(s..e)
                .filter(|element| element.range.start.0 >= start && element.range.start.0 < end)
                .map(|element| Interval {
                    start: element.range.start.0,
                    end: element.range.end.0,
                    data: element.value.clone(),
                })
                .collect()
        } else {
            self.pending
                .iter()
                .filter(|i| i.start >= start && i.start < end)
                .cloned()
                .collect()
        }
    }

    /// Returns the end time of the last interval in the timeline.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Call this once after all intervals are added to optimize lookups.
    pub fn finalize(&mut self) {
        self.ensure_tree();
    }
}
