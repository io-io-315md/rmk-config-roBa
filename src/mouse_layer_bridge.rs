use rmk::event::{ActionEvent, Axis, PointingEvent, publish_event};
use rmk::macros::processor;
use rmk::types::action::Action;
use rmk::types::keycode::{HidKeyCode, KeyCode};

const TRACKBALL_DEVICE_ID: u8 = 0;
const LARGE_MOTION_DEVICE_ID: u8 = 1;
const BACK_BUTTON_DISTANCE: i32 = 10;

#[processor(subscribe = [PointingEvent, ActionEvent])]
pub struct MouseLayerBridge;

impl MouseLayerBridge {
    pub const fn new() -> Self {
        Self
    }

    async fn on_pointing_event(&mut self, event: PointingEvent) {
        if event.device_id != TRACKBALL_DEVICE_ID {
            return;
        }

        let distance = event
            .axes
            .iter()
            .filter(|axis| matches!(axis.axis, Axis::X | Axis::Y))
            .map(|axis| i32::from(axis.value).abs())
            .sum::<i32>();

        if distance > BACK_BUTTON_DISTANCE {
            let mut large_motion = event;
            large_motion.device_id = LARGE_MOTION_DEVICE_ID;
            publish_event(large_motion);
        }
    }

    async fn on_action_event(&mut self, event: ActionEvent) {
        if event.keyboard_event.pressed
            && event.action == Action::Key(KeyCode::Hid(HidKeyCode::MouseBtn4))
        {
            // Mouse button 4 itself is considered a mouse key by RMK. Publish a
            // keyboard-only action event so both automatic mouse layers exit
            // after Enter sends the browser Back command.
            publish_event(ActionEvent {
                action: Action::Key(KeyCode::Hid(HidKeyCode::Enter)),
                keyboard_event: event.keyboard_event,
            });
        }
    }
}

