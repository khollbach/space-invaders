use nrf52833_hal::{
    gpio::{
        p0::{P0_11, P0_15, P0_19, P0_21, P0_22, P0_24, P0_28, P0_30, P0_31},
        p1::P1_05,
        Disconnected, Level, Output, Pin, PushPull,
    },
    prelude::*,
};
use void::ResultVoidExt;

/// The display is a square grid with this many rows and columns.
pub const DISPLAY_SIZE: i8 = 5;

/// 5x5 grid of booleans.
pub type BoolGrid = [[bool; DISPLAY_SIZE as usize]; DISPLAY_SIZE as usize];

/// How long to display each row, before switching to the next row. Call the
/// `Display::update` method this often, in microseconds.
//
// We picked 800us based on suggestions in "Everything I've Learnt About LEDs",
// by Mike Harrison:
// https://www.youtube.com/watch?v=5SQt1f4PsRU
//
// He suggests 250 Hz (= 4 ms) cycle time for dimming a single LED. That is,
// every 4 ms, the LED first turns on (e.g. for 3 ms) and then off (e.g. for 1
// ms).
//
// So, we've decided to cycle through all 5 rows once every 4 ms. This means
// spending 800 us on each row.
pub const DISPLAY_TIMER_US: u32 = 800;

/// The 5x5 LED display has only 10 pins -- one for each row and column. To turn
/// on a specific LED ("pixel"), we must set the corresponding row *high* and
/// corresponding column *low*. In particular, this means we can't display an
/// arbitrary pattern of 25 pixels at a single instant in time.
///
/// We're expected to "strobe" the rows, so that only one row is actually lit at
/// a time. But it happens so fast, that the human eye sees them all
/// continuously illuminated.
pub struct Display {
    rows: [Pin<Output<PushPull>>; DISPLAY_SIZE as usize],
    cols: [Pin<Output<PushPull>>; DISPLAY_SIZE as usize],
    display_buffer: BoolGrid,
    curr_row: i8,
}

impl Display {
    pub fn new(
        row1: P0_21<Disconnected>,
        row2: P0_22<Disconnected>,
        row3: P0_15<Disconnected>,
        row4: P0_24<Disconnected>,
        row5: P0_19<Disconnected>,

        col1: P0_28<Disconnected>,
        col2: P0_11<Disconnected>,
        col3: P0_31<Disconnected>,
        col4: P1_05<Disconnected>,
        col5: P0_30<Disconnected>,
    ) -> Self {
        Self {
            rows: [
                row1.into_push_pull_output(Level::Low).degrade(),
                row2.into_push_pull_output(Level::Low).degrade(),
                row3.into_push_pull_output(Level::Low).degrade(),
                row4.into_push_pull_output(Level::Low).degrade(),
                row5.into_push_pull_output(Level::Low).degrade(),
            ],
            cols: [
                col1.into_push_pull_output(Level::High).degrade(),
                col2.into_push_pull_output(Level::High).degrade(),
                col3.into_push_pull_output(Level::High).degrade(),
                col4.into_push_pull_output(Level::High).degrade(),
                col5.into_push_pull_output(Level::High).degrade(),
            ],
            display_buffer: [[false; 5]; 5],
            curr_row: DISPLAY_SIZE - 1,
        }
    }

    /// You must call this method every `DISPLAY_TIMER_US` microseconds in order
    /// for it to work correctly.
    ///
    /// The `update_display_buffer` function only gets called during every fifth
    /// update.
    pub fn update<F>(&mut self, update_display_buffer: F)
    where
        F: FnOnce(&mut BoolGrid),
    {
        // Clear the previous row.
        self.rows[self.curr_row as usize].set_low().void_unwrap();

        self.curr_row += 1;
        self.curr_row %= DISPLAY_SIZE;

        if self.curr_row == 0 {
            update_display_buffer(&mut self.display_buffer);
        }

        for col in 0..DISPLAY_SIZE {
            let state = if self.display_buffer[self.curr_row as usize][col as usize] {
                PinState::Low // on
            } else {
                PinState::High // off
            };
            self.cols[col as usize].set_state(state).void_unwrap();
        }

        // Display the current row.
        self.rows[self.curr_row as usize].set_high().void_unwrap();
    }
}
