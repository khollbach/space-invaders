use rtt_target::rprintln;

use super::ButtonState;

/// How often you should call the Debouncer's update method, in microseconds.
pub const DEBOUNCER_TIMER_US: u32 = 1_000;

/// After this many consecutive `update` calls with the same value, we commit to
/// the new value.
const NUM_CONSECUTIVE_SAMPLES: u8 = 5;

/// Hardware buttons are apparently notorious for "bouncing".
///
/// For more details, see https://www.ganssle.com/debouncing.htm
pub struct Debouncer {
    state: ButtonState,
    count: u8,
}

impl Debouncer {
    pub const fn new(initial_state: ButtonState) -> Self {
        Self {
            state: initial_state,
            count: 0,
        }
    }

    /// Filter out transient state changes.
    ///
    /// Returns `Some` whenever the state "actually" changes.
    ///
    /// This uses the debouncing approach described in
    /// https://www.ganssle.com/debouncing-pt2.htm
    /// (section: "A Counting Algorithm").
    pub fn update(&mut self, new_state: ButtonState) -> Option<ButtonState> {
        let mut changed = false;

        if new_state != self.state {
            self.count += 1;
            if self.count >= NUM_CONSECUTIVE_SAMPLES {
                self.state = new_state;
                self.count = 0;
                changed = true;
            }
        } else {
            if self.count > 0 {
                rprintln!("debounce {}", self.count);
            }
            self.count = 0;
        }

        if changed {
            Some(self.state)
        } else {
            None
        }
    }
}
