use super::{playing::Playing, start_animation::StartAnimation, GamePhase, Phase};
use crate::display::{BoolGrid, DISPLAY_SIZE};

pub struct WinAnimation {
    game_state: Playing,
    num_updates: u32,
}

impl WinAnimation {
    pub fn new(game_state: Playing) -> Self {
        Self {
            game_state,
            num_updates: 0,
        }
    }
}

impl GamePhase for WinAnimation {
    fn display(&self, display_buffer: &mut BoolGrid) {
        // For a brief moment, show the state of the game when the last enemy
        // was destroyed. Otherwise, it'd be kinda jarring.
        if self.num_updates < 10 {
            self.game_state.display(display_buffer);
            return;
        }

        // After 4 "sweep" effects, clear the screen for a moment.
        if self.num_updates >= 50 {
            *display_buffer = [[false; 5]; 5];
            return;
        }

        let tick = self.num_updates as usize % 10;

        for row in 0..DISPLAY_SIZE as usize {
            for col in 0..DISPLAY_SIZE as usize {
                let sum = row + col;

                // The diagonal "sweep" effect is two pixels wide.
                let leading_edge = sum == tick;
                let trailing_edge = sum + 1 == tick;

                display_buffer[row][col] = leading_edge || trailing_edge;
            }
        }
    }

    fn update_timer_ms(&self) -> u32 {
        50
    }

    fn update(&mut self) -> Option<Phase> {
        self.num_updates += 1;

        if self.num_updates < 70 {
            None
        } else {
            Some(Phase::StartAnimation(StartAnimation::new()))
        }
    }
}
