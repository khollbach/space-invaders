use core::{
    arch::asm,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};

use nrf52833_hal::{
    gpio::{Disconnected, Floating, Input, Pin},
    prelude::InputPin,
};
use void::ResultVoidExt;

use self::debouncer::Debouncer;

mod debouncer;

pub struct Button {
    pin: Pin<Input<Floating>>,
    state: ButtonState,
    debouncer: Debouncer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    NotPressed,
    Pressed,
}

impl Button {
    pub fn new(pin: Pin<Disconnected>) -> Self {
        let pin = pin.into_floating_input();

        // Configuring the pin and then immediately reading from it sometimes
        // produces the wrong value. This leads us to believe the button is
        // pressed, even when it's not.
        //
        // Pausing for one CPU cycle seems to fix the issue.
        //
        // Possible explanation here:
        // https://devzone.nordicsemi.com/f/nordic-q-a/36881/best-practice---delay-between-setting-pin-as-input-with-pull-up-pull-down-and-reading-the-pin/141580
        // additional context here: (page 16)
        // https://infocenter.nordicsemi.com/pdf/nRF52_Series_Migration.pdf
        // and here: (at the bottom of page 100)
        // https://infocenter.nordicsemi.com/pdf/nRF52833_PS_v1.6.pdf
        //
        // One issue with this explanation is that the GPIO peripheral is
        // connected to the *AHB*, not the APB. As far as I can tell, the AHB
        // doesn't buffer writes.

        // Insert a "nop" between configuring the pin and reading it.
        compiler_fence(SeqCst);
        // SAFETY: I'll be honest, I haven't read the
        // [rules](https://doc.rust-lang.org/reference/inline-assembly.html#rules-for-inline-assembly),
        // but inserting a single "no-operation" instruction seems like a pretty
        // harmless use of inline assembly, so I'll assume there's no undefined
        // behaviour here.
        unsafe { asm!("nop") };
        compiler_fence(SeqCst);

        let initial_state = read_state(&pin);

        Self {
            pin,
            state: initial_state,
            debouncer: Debouncer::new(initial_state),
        }
    }

    pub fn state(&self) -> ButtonState {
        self.state
    }

    /// Returns `Some` whenever `state` changes.
    ///
    /// You must call this every `BUTTON_TIMER_US` microseconds for it to work
    /// correctly.
    pub fn update(&mut self) -> Option<ButtonState> {
        match self.debouncer.update(read_state(&self.pin)) {
            None => None,
            Some(new_state) => {
                if new_state == self.state {
                    None
                } else {
                    self.state = new_state;
                    Some(new_state)
                }
            }
        }
    }

    /// Force the button into a certain state.
    ///
    /// For example, if the button is physically pressed, you may force it to be
    /// released. Later, when the button is physically released, `update` will
    /// return `None`, since the button's `state` hasn't changed.
    pub fn force_state_change(&mut self, new_state: ButtonState) {
        self.state = new_state;
    }
}

/// You must call the Button's `update` method this often, so that the
/// debouncing algorithm works correctly.
pub use self::debouncer::DEBOUNCER_TIMER_US as BUTTON_TIMER_US;

fn read_state(pin: &Pin<Input<Floating>>) -> ButtonState {
    // Note: low = pressed.
    if pin.is_low().void_unwrap() {
        ButtonState::Pressed
    } else {
        ButtonState::NotPressed
    }
}

impl ButtonState {
    pub fn is_pressed(self) -> bool {
        match self {
            Self::NotPressed => false,
            Self::Pressed => true,
        }
    }
}
