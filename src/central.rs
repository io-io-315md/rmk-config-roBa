#![no_main]
#![no_std]

mod pointing_mode_controller;

use rmk::macros::rmk_central;

#[rmk_central]
mod keyboard_central {
    #[register_processor(event)]
    fn pointing_mode_controller() -> crate::pointing_mode_controller::PointingModeController {
        crate::pointing_mode_controller::PointingModeController::new()
    }
}
