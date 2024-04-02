use super::{loss_animation::LossAnimation, win_animation::WinAnimation, GamePhase, Phase};
use crate::{
    buttons::ButtonAction,
    display::{BoolGrid, DISPLAY_SIZE},
};

#[derive(Debug, Clone)]
pub struct Playing {
    pub player_x: i8,
    pub bullets: BoolGrid,
    pub enemies: BoolGrid,
    num_updates: u32,
}

impl Playing {
    pub fn new() -> Self {
        let mut this = Self {
            player_x: DISPLAY_SIZE / 2,
            bullets: [[false; 5]; 5],
            enemies: [
                [true, true, true, true, false], // 4 enemies
                [false, false, false, false, false],
                [false, false, false, false, false],
                [false, false, false, false, false],
                [false, false, false, false, false],
            ],
            num_updates: 0,
        };

        // Move the enemies immediately when the game starts. This gives the
        // player a visual cue that they can now act.
        let outcome = this.update();
        debug_assert!(outcome.is_none());

        this
    }

    /// Perform an action in response to player input.
    pub fn player_action(&mut self, action: ButtonAction) {
        match action {
            ButtonAction::Fire => {
                let row = DISPLAY_SIZE as usize - 2;
                let col = self.player_x as usize;

                if self.enemies[row][col] {
                    // Edge-case: the bullet immediately hits an enemy.
                    self.enemies[row][col] = false;
                } else {
                    // Fire a bullet.
                    self.bullets[row][col] = true;
                }
            }
            ButtonAction::Left => {
                self.player_x -= 1;
            }
            ButtonAction::Right => {
                self.player_x += 1;
            }
        }
        self.player_x = self.player_x.clamp(0, DISPLAY_SIZE - 1);
    }
}

impl GamePhase for Playing {
    fn display(&self, display_buffer: &mut BoolGrid) {
        *display_buffer = [[false; 5]; 5];

        for row in 0..DISPLAY_SIZE as usize {
            for col in 0..DISPLAY_SIZE as usize {
                if self.bullets[row][col] || self.enemies[row][col] {
                    display_buffer[row][col] = true;
                }
            }
        }

        display_buffer[DISPLAY_SIZE as usize - 1][self.player_x as usize] = true;
    }

    fn update_timer_ms(&self) -> u32 {
        500
    }

    fn update(&mut self) -> Option<Phase> {
        self.move_enemies();
        self.move_bullets();
        self.num_updates += 1;
        self.check_gameover()
    }
}

impl Playing {
    fn move_enemies(&mut self) {
        // Enemies move every other tick.
        if self.num_updates % 2 != 0 {
            return;
        }

        let cycle = self.num_updates / 2;
        let (drow, dcol) = match cycle % 4 {
            0 => (0, 1),     // right
            1 | 3 => (1, 0), // down
            2 => (0, -1),    // left
            _ => unreachable!(),
        };

        let mut out = BoolGrid::default();
        for row in 0..DISPLAY_SIZE {
            for col in 0..DISPLAY_SIZE {
                if self.enemies[row as usize][col as usize] {
                    let r = row + drow;
                    let c = col + dcol;
                    assert!(in_bounds(r, c), "enemy moved off screen ({r}, {c})");
                    out[r as usize][c as usize] = true;
                }
            }
        }
        self.enemies = out;

        self.check_collision();
    }

    fn move_bullets(&mut self) {
        // Bullets in the top row simply disappear.
        for col in 0..DISPLAY_SIZE as usize {
            self.bullets[0][col] = false;
        }

        for row in 1..DISPLAY_SIZE as usize {
            for col in 0..DISPLAY_SIZE as usize {
                if self.bullets[row][col] {
                    self.bullets[row][col] = false;
                    self.bullets[row - 1][col] = true;
                }
            }
        }

        self.check_collision();
    }

    /// If a bullet overlaps an enemy, both are destroyed.
    fn check_collision(&mut self) {
        for row in 0..DISPLAY_SIZE as usize {
            for col in 0..DISPLAY_SIZE as usize {
                if self.bullets[row][col] && self.enemies[row][col] {
                    self.bullets[row][col] = false;
                    self.enemies[row][col] = false;
                }
            }
        }
    }

    fn check_gameover(&self) -> Option<Phase> {
        // Did the enemies reach the bottom?
        for col in 0..DISPLAY_SIZE as usize {
            if self.enemies[DISPLAY_SIZE as usize - 1][col] {
                return Some(Phase::LossAnimation(LossAnimation::new(self.clone())));
            }
        }

        // Are there any enemies remaining?
        for row in 0..DISPLAY_SIZE as usize {
            for col in 0..DISPLAY_SIZE as usize {
                if self.enemies[row][col] {
                    return None;
                }
            }
        }

        Some(Phase::WinAnimation(WinAnimation::new(self.clone())))
    }
}

fn in_bounds(row: i8, col: i8) -> bool {
    let row = 0 <= row && row < DISPLAY_SIZE;
    let col = 0 <= col && col < DISPLAY_SIZE;
    row && col
}
