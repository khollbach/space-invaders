use super::{playing::Playing, GamePhase, Phase};
use crate::display::{BoolGrid, DISPLAY_SIZE};

pub struct StartAnimation {
    num_updates: u32,
}

impl StartAnimation {
    pub fn new() -> Self {
        Self { num_updates: 0 }
    }
}

impl GamePhase for StartAnimation {
    fn display(&self, display_buffer: &mut BoolGrid) {
        *display_buffer = [[false; 5]; 5];

        // Blink player.
        if self.num_updates % 2 == 0 || self.num_updates >= 4 {
            let row = DISPLAY_SIZE - 1;
            let col = DISPLAY_SIZE / 2;
            display_buffer[row as usize][col as usize] = true;
        }

        if self.num_updates >= 8 {
            // Blink enemies.
            if self.num_updates % 2 == 0 || self.num_updates >= 12 {
                // 4 enemies.
                for col in 0..DISPLAY_SIZE - 1 {
                    display_buffer[0][col as usize] = true;
                }
            }
        }
    }

    fn update_timer_ms(&self) -> u32 {
        250
    }

    fn update(&mut self) -> Option<Phase> {
        self.num_updates += 1;

        if self.num_updates < 16 {
            None
        } else {
            Some(Phase::Playing(Playing::new()))
        }
    }
}
