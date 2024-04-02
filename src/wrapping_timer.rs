use core::ops::Deref;

use nrf52833_hal::pac::timer0;

/// A timer that counts upwards from 0 through u32::MAX and then wraps back
/// around to 0. Ticks once every microsecond.
pub struct WrappingTimer<T: Deref<Target = timer0::RegisterBlock>> {
    timer: T,
}

impl<T: Deref<Target = timer0::RegisterBlock>> WrappingTimer<T> {
    /// Start the timer.
    pub fn new(timer: T) -> Self {
        timer.tasks_stop.write(|w| w.tasks_stop().trigger());

        timer.shorts.reset();
        timer.intenclr.write(|w| {
            w.compare0().clear();
            w.compare1().clear();
            w.compare2().clear();
            w.compare3().clear();
            w.compare4().clear();
            w.compare5().clear()
        });

        timer.mode.write(|w| w.mode().timer());
        timer.bitmode.write(|w| w.bitmode()._32bit());

        // From the docs, `frequency = 16 MHz / 2^prescaler`.
        // So we use a prescaler of 4 to get a 1 MHz timer.
        timer.prescaler.write(|w| w.prescaler().variant(4));

        timer.tasks_clear.write(|w| w.tasks_clear().trigger());
        timer.tasks_start.write(|w| w.tasks_start().trigger());

        Self { timer }
    }

    /// Return the current timestamp, in microseconds.
    pub fn curr_time_us(&self) -> u32 {
        self.timer.tasks_capture[0].write(|w| w.tasks_capture().trigger());
        self.timer.cc[0].read().cc().bits()
    }

    /// How long, in microseconds, has it been since the given timestamp?
    ///
    /// Note that the answer will wrap, if >= 2^32 microseconds have elapsed
    /// since the given timestamp. 2^32 microseconds is about an hour -- so
    /// don't use this to measure durations any longer than that.
    pub fn elapsed_us(&self, earlier_time_us: u32) -> u32 {
        self.curr_time_us().wrapping_sub(earlier_time_us)
    }
}
