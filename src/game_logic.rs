use self::{
    loss_animation::LossAnimation, playing::Playing, start_animation::StartAnimation,
    win_animation::WinAnimation,
};
use crate::{buttons::ButtonAction, display::BoolGrid};

mod loss_animation;
mod playing;
mod start_animation;
mod win_animation;

/// How often to call Game::update, in microseconds.
//
// The code below depends on this being 1 millisecond.
pub const GAME_UPDATE_TIMER_US: u32 = 1_000;

pub struct Game {
    phase: Phase,
    num_updates: u32,
}

enum Phase {
    StartAnimation(StartAnimation),
    Playing(Playing),
    LossAnimation(LossAnimation),
    WinAnimation(WinAnimation),
}

/// Common functionality of various phases of the game.
trait GamePhase {
    fn display(&self, display_buffer: &mut BoolGrid);

    /// How often to call `update`, in milliseconds. Must be non-zero.
    fn update_timer_ms(&self) -> u32;

    fn update(&mut self) -> Option<Phase>;
}

impl Game {
    pub fn new() -> Self {
        Self {
            phase: Phase::StartAnimation(StartAnimation::new()),
            num_updates: 0,
        }
    }

    pub fn display(&self, display_buffer: &mut BoolGrid) {
        self.game_phase().display(display_buffer);
    }

    pub fn player_action(&mut self, action: ButtonAction) {
        if let Phase::Playing(p) = &mut self.phase {
            p.player_action(action);
        }
    }

    pub fn update(&mut self) {
        if self.num_updates % self.game_phase().update_timer_ms() == 0 {
            if let Some(new_phase) = self.game_phase_mut().update() {
                self.phase = new_phase;
                self.num_updates = 0;
            }
        }
        self.num_updates += 1;
    }

    fn game_phase(&self) -> &dyn GamePhase {
        match &self.phase {
            Phase::StartAnimation(s) => s,
            Phase::Playing(p) => p,
            Phase::LossAnimation(l) => l,
            Phase::WinAnimation(w) => w,
        }
    }

    fn game_phase_mut(&mut self) -> &mut dyn GamePhase {
        match &mut self.phase {
            Phase::StartAnimation(s) => s,
            Phase::Playing(p) => p,
            Phase::LossAnimation(l) => l,
            Phase::WinAnimation(w) => w,
        }
    }
}
