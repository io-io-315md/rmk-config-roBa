use rmk::event::{LayerChangeEvent, PointingProcessorEvent, publish_event};
use rmk::input_device::pointing::{CursorConfig, PointingMode, ScrollConfig};
use rmk::macros::processor;

const TRACKBALL_DEVICE_ID: u8 = 0;
const SCROLL_LAYER: u8 = 5;

#[processor(subscribe = [LayerChangeEvent])]
pub struct PointingModeController;

impl PointingModeController {
    pub const fn new() -> Self {
        Self
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        let mode = if event.0 == SCROLL_LAYER {
            PointingMode::Scroll(ScrollConfig {
                multiplier_x: 1,
                divisor_x: 80,
                multiplier_y: 1,
                divisor_y: 80,
                invert_x: false,
                invert_y: false,
            })
        } else {
            PointingMode::Cursor(CursorConfig::default())
        };

        publish_event(PointingProcessorEvent {
            device_id: TRACKBALL_DEVICE_ID,
            mode,
        });
    }
}

