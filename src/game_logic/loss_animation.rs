use super::{playing::Playing, start_animation::StartAnimation, GamePhase, Phase};
use crate::display::{BoolGrid, DISPLAY_SIZE};

pub struct LossAnimation {
    game_state: Playing,
    num_updates: u32,
}

impl LossAnimation {
    pub fn new(game_state: Playing) -> Self {
        Self {
            game_state,
            num_updates: 0,
        }
    }
}

impl GamePhase for LossAnimation {
    fn display(&self, display_buffer: &mut BoolGrid) {
        *display_buffer = [[false; 5]; 5];

        // Blank screen briefly before restarting.
        if self.num_updates >= 6 {
            return;
        }

        self.game_state.display(display_buffer);

        // Blink enemies in the bottom row.
        let show = self.num_updates % 2 == 0 || self.num_updates >= 4;
        for col in 0..DISPLAY_SIZE as usize {
            let row = DISPLAY_SIZE as usize - 1;
            if self.game_state.enemies[row][col] {
                display_buffer[row][col] = show;
            }
        }
    }

    fn update_timer_ms(&self) -> u32 {
        500
    }

    fn update(&mut self) -> Option<Phase> {
        self.num_updates += 1;

        if self.num_updates < 7 {
            None
        } else {
            Some(Phase::StartAnimation(StartAnimation::new()))
        }
    }
}
