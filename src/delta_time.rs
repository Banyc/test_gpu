use std::time::{Duration, Instant};

use num_traits::CheckedSub;

#[derive(Debug, Clone)]
pub struct DeltaValue<T> {
    prev: T,
}
impl<T> DeltaValue<T> {
    pub fn new(time: T) -> Self {
        Self { prev: time }
    }
}
impl<T> DeltaValue<T>
where
    T: CheckedSub,
{
    pub fn update(&mut self, now: T) -> Option<T> {
        let delta = now.checked_sub(&self.prev)?;
        self.prev = now;
        Some(delta)
    }
}

#[derive(Debug, Clone)]
pub struct DeltaTime {
    prev_time: Instant,
    prev_delta: Option<Duration>,
}
impl DeltaTime {
    pub fn new(time: Instant) -> Self {
        Self {
            prev_time: time,
            prev_delta: None,
        }
    }

    pub fn update(&mut self, now: Instant) {
        let delta = now - self.prev_time;
        self.prev_time = now;
        self.prev_delta = Some(delta);
    }

    pub fn delta(&self) -> Option<Duration> {
        self.prev_delta
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_delta_time() {
        let mut d = DeltaTime::new(Instant::now());
        d.update(Instant::now());
    }
}
