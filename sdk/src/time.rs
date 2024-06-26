use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Timer(Instant);

impl Default for Timer {
    fn default() -> Self {
        Self(Instant::now())
    }
}

#[allow(dead_code)]
impl Timer {
    pub fn once(&self, duration: Duration) -> bool {
        let now = Instant::now();
        let delta = now - self.0;
        delta > duration
    }

    pub fn elapsed(&mut self, duration: Duration) -> bool {
        if self.0.elapsed() > duration {
            self.0 = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.0 = Instant::now();
    }
}

#[derive(Default)]
pub struct Timers<T>(pub HashMap<T, Timer>)
where
    T: std::hash::Hash + std::cmp::Eq + std::cmp::PartialEq;

impl<T> Timers<T>
where
    T: std::hash::Hash + std::cmp::Eq + std::cmp::PartialEq,
{
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn elapsed(&mut self, tag: T, duration: Duration) -> bool {
        match self.0.get_mut(&tag) {
            Some(timer) => {
                if timer.elapsed(duration) {
                    self.0.insert(tag, Timer(Instant::now()));
                    return true;
                }

                false
            }
            None => {
                self.0.insert(tag, Timer(Instant::now()));
                false
            }
        }
    }

    pub fn reset(&mut self, tag: T) {
        match self.0.get_mut(&tag) {
            Some(timer) => {
                timer.reset();
            }
            None => {
                self.0.insert(tag, Timer(Instant::now()));
            }
        }
    }

    pub fn add(&mut self, tag: T) {
        self.0.insert(tag, Timer::default());
    }

    pub fn get(&self, tag: T) -> Option<&Timer> {
        self.0.get(&tag)
    }

    pub fn get_mut(&mut self, tag: T) -> Option<&mut Timer> {
        self.0.get_mut(&tag)
    }
}

#[cfg(test)]
mod time {
    use super::*;
    use std::time::Duration;

    #[test]
    #[allow(unused_assignments)]
    fn timer_once() {
        let timer = Timer::default();
        let mut data = false;

        loop {
            if timer.once(Duration::from_secs(2)) {
                data = true;
                break;
            }
        }
        assert!(data);
    }

    #[test]
    fn timer_every() {
        let start = Instant::now();
        let mut timer = Timer::default();
        let mut count = 0;

        loop {
            if timer.elapsed(Duration::from_secs(1)) {
                count += 1;
            }

            if count >= 5 {
                break;
            }
        }

        let delta = Instant::now() - start;
        assert_eq!(count, 5);
        assert!(delta >= Duration::from_secs(5))
    }
}
