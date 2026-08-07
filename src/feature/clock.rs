//! A source of the current time, so time-dependent logic (like a cache's
//! time to live) can be tested without sleeping.

use std::time::Instant;

/// A source of the current time.
pub trait Clock: 'static {
    /// The current time.
    fn now(&self) -> Instant;
}

/// The real, monotonic system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A clock that only moves when a test moves it.
#[cfg(test)]
pub mod fake {
    use super::Clock;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    /// A clock that only moves when a test calls [`FixedClock::advance`].
    /// Cloneable so a test can keep a handle after handing one to the code
    /// under test.
    #[derive(Clone)]
    pub struct FixedClock {
        now: Rc<Cell<Instant>>,
    }

    impl FixedClock {
        /// A clock fixed at the current moment.
        pub fn new() -> Self {
            Self {
                now: Rc::new(Cell::new(Instant::now())),
            }
        }

        /// Moves the clock forward by `amount`.
        pub fn advance(&self, amount: Duration) {
            self.now.set(self.now.get() + amount);
        }
    }

    impl Default for FixedClock {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Clock for FixedClock {
        fn now(&self) -> Instant {
            self.now.get()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn a_fixed_clock_only_moves_when_advanced() {
            let clock = FixedClock::new();
            let before = clock.now();

            clock.advance(Duration::from_secs(60));

            assert_eq!(clock.now(), before + Duration::from_secs(60));
        }

        #[test]
        fn advancing_a_fixed_clock_is_visible_through_a_clone() {
            let clock = FixedClock::new();
            let clone = clock.clone();

            clock.advance(Duration::from_secs(60));

            assert_eq!(clone.now(), clock.now());
        }
    }
}
