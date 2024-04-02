#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtic::app;

mod buttons;
mod display;
mod game_logic;
mod wrapping_timer;

#[app(device = nrf52833_hal::pac)]
mod app {
    use nrf52833_hal::{
        gpio::{p0, p1},
        pac,
        prelude::*,
        timer::Periodic,
        Timer,
    };
    use rtt_target::rprintln;

    use crate::{
        buttons::{button::BUTTON_TIMER_US, Buttons},
        display::{Display, DISPLAY_TIMER_US},
        game_logic::{Game, GAME_UPDATE_TIMER_US},
    };

    #[shared]
    struct Shared {
        game: Game,
    }

    #[local]
    struct Local {
        //
        // update_display
        //
        display_timer: Timer<pac::TIMER0, Periodic>,
        display: Display,

        //
        // check_buttons
        //
        button_timer: Timer<pac::TIMER1, Periodic>,
        buttons: Buttons<pac::TIMER2>,

        //
        // game_update
        //
        game_update_timer: Timer<pac::TIMER3, Periodic>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_target::rtt_init_print!();
        rprintln!("hello world");

        let mut display_timer = Timer::periodic(cx.device.TIMER0);
        display_timer.enable_interrupt();
        display_timer.start(DISPLAY_TIMER_US);

        let mut button_timer = Timer::periodic(cx.device.TIMER1);
        button_timer.enable_interrupt();
        button_timer.start(BUTTON_TIMER_US);

        let mut game_update_timer = Timer::periodic(cx.device.TIMER3);
        game_update_timer.enable_interrupt();
        game_update_timer.start(GAME_UPDATE_TIMER_US);

        let p0 = p0::Parts::new(cx.device.P0);
        let p1 = p1::Parts::new(cx.device.P1);

        (
            Shared { game: Game::new() },
            Local {
                display_timer,
                display: Display::new(
                    p0.p0_21, p0.p0_22, p0.p0_15, p0.p0_24, p0.p0_19, p0.p0_28, p0.p0_11, p0.p0_31,
                    p1.p1_05, p0.p0_30,
                ),

                button_timer,
                buttons: Buttons::new(p0.p0_14, p0.p0_23, cx.device.TIMER2),

                game_update_timer,
            },
        )
    }

    #[task(binds = TIMER0, shared = [game], local = [display_timer, display])]
    fn update_display(mut cx: update_display::Context) {
        cx.local.display.update(|display_buffer| {
            cx.shared.game.lock(|game| {
                game.display(display_buffer);
            });
        });

        cx.local.display_timer.reset_event();
    }

    #[task(binds = TIMER1, shared = [game], local = [button_timer, buttons])]
    fn check_buttons(mut cx: check_buttons::Context) {
        if let Some(action) = cx.local.buttons.update() {
            cx.shared.game.lock(|game| {
                game.player_action(action);
            });
        }

        cx.local.button_timer.reset_event();
    }

    #[task(binds = TIMER3, shared = [game], local = [game_update_timer])]
    fn game_update(mut cx: game_update::Context) {
        cx.shared.game.lock(|game| {
            game.update();
        });

        cx.local.game_update_timer.reset_event();
    }
}
