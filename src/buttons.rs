use core::ops::Deref;

use nrf52833_hal::{
    gpio::{
        p0::{P0_14, P0_23},
        Disconnected,
    },
    pac::timer0,
};

use self::button::{Button, ButtonState};
use crate::wrapping_timer::WrappingTimer;

pub mod button;

/// When the user presses a button, how long do we wait to see if they press the
/// other button as well? In microseconds.
const PRESS_AND_HOLD_TIMEOUT_US: u32 = 100_000; // 100,000 us = 100 ms

/// Input handling for the Space Invaders game.
///
/// Press A or B to move left or right. Press both at once to fire.
pub struct Buttons<T: Deref<Target = timer0::RegisterBlock>> {
    buttons: [Button; 2],
    time_source: WrappingTimer<T>,
    /// If either button is held, this is the timestamp of when it was
    /// initially pressed. At most one button is held at a time.
    held_since: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    Left,
    Right,
    Fire,
}

impl<T: Deref<Target = timer0::RegisterBlock>> Buttons<T> {
    pub fn new(a: P0_14<Disconnected>, b: P0_23<Disconnected>, time_source: T) -> Self {
        let mut this = Self {
            buttons: [Button::new(a.degrade()), Button::new(b.degrade())],
            time_source: WrappingTimer::new(time_source),
            held_since: None,
        };

        // The `update` logic is simpler if we can assume both buttons start
        // "not-pressed".
        for b in &mut this.buttons {
            if b.state().is_pressed() {
                b.force_state_change(ButtonState::NotPressed);
            }
        }

        this
    }

    /// You must call this every `button::BUTTON_TIMER_US` microseconds for it
    /// to work correctly.
    pub fn update(&mut self) -> Option<ButtonAction> {
        debug_assert!({
            let both_pressed = self.buttons.iter().all(|b| b.state().is_pressed());
            !both_pressed
        });

        let actions = [
            self.check_timeout(),
            self.update_button(0),
            self.update_button(1),
        ];

        debug_assert!({
            let num_actions = actions.iter().flatten().count();
            num_actions <= 1
        });

        actions.into_iter().flatten().next()
    }

    /// Check if a button has been held down for more than a short moment. If
    /// so, release the button.
    ///
    /// This makes movement feel more responsive. The player probably expects
    /// actions to happen when they *press* buttons, not when they release them.
    ///
    /// Instead of reacting *immediately* on-press, we wait briefly to see if
    /// the user presses the other button as well. That way we can still detect
    /// the user pressing both buttons "at the same time".
    fn check_timeout(&mut self) -> Option<ButtonAction> {
        if let Some(t) = self.held_since {
            if self.time_source.elapsed_us(t) >= PRESS_AND_HOLD_TIMEOUT_US {
                // Release whichever button was held.
                for i in 0..2 {
                    if self.buttons[i].state().is_pressed() {
                        self.buttons[i].force_state_change(ButtonState::NotPressed);
                        self.held_since = None;
                        return Some(ButtonAction::left_right(i));
                    }
                }
                debug_assert!(false);
            }
        }
        None
    }

    /// Check a button for state changes.
    fn update_button(&mut self, i: usize) -> Option<ButtonAction> {
        match self.buttons[i].update() {
            None => None,
            Some(ButtonState::Pressed) => {
                // Is the other button also pressed?
                if self.buttons[1 - i].state().is_pressed() {
                    // Treat both buttons as released, so the player doesn't
                    // move they physically release the buttons.
                    self.buttons[i].force_state_change(ButtonState::NotPressed);
                    self.buttons[1 - i].force_state_change(ButtonState::NotPressed);
                    self.held_since = None;
                    Some(ButtonAction::Fire)
                } else {
                    self.held_since = Some(self.time_source.curr_time_us());
                    None
                }
            }
            Some(ButtonState::NotPressed) => {
                self.held_since = None;
                Some(ButtonAction::left_right(i))
            }
        }
    }
}

impl ButtonAction {
    fn left_right(i: usize) -> Self {
        match i {
            0 => Self::Left,
            1 => Self::Right,
            _ => panic!("buttons are indexed with 0 or 1, got: {i}"),
        }
    }
}
