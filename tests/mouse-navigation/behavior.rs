#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    use rmk::channel::USB_REPORT_CHANNEL;
    use rmk::config::{AutoMouseLayerConfig, BehaviorConfig, PositionalConfig};
    use rmk::core_traits::Runnable;
    use rmk::embassy_futures::select::select;
    use rmk::embassy_time::{Duration, MockDriver, Timer};
    use rmk::event::{Axis, AxisEvent, AxisValType, KeyboardEvent, PointingEvent, publish_event};
    use rmk::hid::Report;
    use rmk::keyboard::Keyboard;
    use rmk::keyboard::combo::{Combo, ComboConfig};
    use rmk::keymap::{KeyMap, KeymapData};
    use rmk::types::action::{Action, KeyAction};
    use rmk::types::connection::UsbState;
    use rmk::types::keycode::{HidKeyCode, KeyCode};
    use rmk::AutoMouseLayerRunner;

    const J: (u8, u8) = (1, 7);
    const K: (u8, u8) = (1, 8);
    const L: (u8, u8) = (1, 9);
    const ENTER: (u8, u8) = (3, 9);

    fn action(name: &str) -> KeyAction {
        if name == "_" {
            KeyAction::Transparent
        } else {
            let hid: HidKeyCode = serde_json::from_value(serde_json::Value::String(name.into())).unwrap();
            KeyAction::Single(Action::Key(KeyCode::Hid(hid)))
        }
    }

    fn fixture() -> (KeymapData<4, 11, 8>, BehaviorConfig) {
        let config = rmk_config::KeyboardTomlConfig::new_from_toml_path(env!("KEYBOARD_TOML_PATH"));
        let layout = config.keymap().unwrap();
        let resolved = config.behavior().unwrap();
        let mut keys = [[[KeyAction::Transparent; 11]; 4]; 8];
        // Read the actual bindings, including any higher-layer Enter override.
        for (layer, map) in keys.iter_mut().enumerate() {
            for (row, col) in [J, K, L, ENTER] {
                map[row as usize][col as usize] = action(&layout.keymap[layer][row as usize][col as usize]);
            }
        }

        let mut behavior = BehaviorConfig::default();
        let combos = resolved.combos.unwrap();
        behavior.combo.timeout = Duration::from_millis(combos.timeout_ms.unwrap_or(50));
        let mut navigation_combos = 0;
        for (index, combo) in combos.combos.iter().enumerate() {
            if matches!(combo.output.as_str(), "MouseBtn4" | "MouseBtn5") {
                behavior.combo.combos[index] = Some(Combo::new(ComboConfig::new(
                    combo.actions.iter().map(|name| action(name)),
                    action(&combo.output),
                    combo.layer,
                )));
                navigation_combos += 1;
            }
        }
        assert_eq!(navigation_combos, 2);
        for entry in resolved.auto_mouse_layer {
            assert!(entry.extra_mouse_keys.is_empty());
            behavior.auto_mouse_layer.push(AutoMouseLayerConfig {
                device_id: entry.device_id,
                target_layer: entry.target_layer,
                timeout: Duration::from_millis(entry.timeout_ms),
                threshold: entry.threshold,
                deactivate_on_key: entry.deactivate_on_key,
                extra_mouse_keys: &[],
                reset_timeout_on_key: entry.reset_timeout_on_key,
            }).unwrap();
        }
        (KeymapData::new(keys), behavior)
    }

    fn key(pos: (u8, u8), pressed: bool) {
        publish_event(KeyboardEvent::key(pos.0, pos.1, pressed));
    }

    async fn motion(keymap: &KeyMap<'_>) {
        publish_event(PointingEvent {
            device_id: 0,
            axes: [
                AxisEvent { typ: AxisValType::Rel, axis: Axis::X, value: 30 },
                AxisEvent { typ: AxisValType::Rel, axis: Axis::Y, value: 0 },
                AxisEvent { typ: AxisValType::Rel, axis: Axis::Z, value: 0 },
            ],
        });
        Timer::after_millis(2).await;
        assert_eq!(keymap.active_layer(), 4);
    }

    fn mouse_report(buttons: u8) {
        let report = USB_REPORT_CHANNEL.try_receive().expect("missing mouse report");
        match report {
            Report::MouseReport(report) => assert_eq!(report.buttons, buttons),
            _ => panic!("unexpected keyboard input during mouse combo"),
        }
    }

    async fn chord(keymap: &KeyMap<'_>, first: (u8, u8), second: (u8, u8), buttons: u8) {
        motion(keymap).await;
        key(first, true);
        Timer::after_millis(10).await;
        assert_eq!(keymap.active_layer(), 4, "first combo key left mouse mode");
        assert!(USB_REPORT_CHANNEL.try_receive().is_err(), "combo key leaked before resolution");
        key(second, true);
        Timer::after_millis(2).await;
        mouse_report(buttons);
        assert_eq!(keymap.active_layer(), 4, "navigation combo left mouse mode");
        key(second, false);
        key(first, false);
        Timer::after_millis(2).await;
        mouse_report(0);
        assert!(USB_REPORT_CHANNEL.try_receive().is_err());
        assert_eq!(keymap.active_layer(), 4);
    }

    async fn typing(keymap: &KeyMap<'_>, pos: (u8, u8), expected: HidKeyCode) {
        key(pos, true);
        Timer::after_millis(70).await;
        match USB_REPORT_CHANNEL.try_receive().expect("missing keyboard report") {
            Report::KeyboardReport(report) => assert!(report.keycodes.contains(&(expected as u8))),
            _ => panic!("normal key unexpectedly sent a mouse button"),
        }
        assert_eq!(keymap.active_layer(), 0);
        key(pos, false);
        Timer::after_millis(2).await;
        match USB_REPORT_CHANNEL.try_receive().expect("missing key release") {
            Report::KeyboardReport(report) => assert_eq!(report.keycodes, [0; 6]),
            _ => panic!("unexpected mouse report on key release"),
        }
    }

    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn run(future: impl Future<Output = ()>) {
        MockDriver::get().reset();
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        let mut future = pin!(future);
        for _ in 0..50_000 {
            if let Poll::Ready(()) = future.as_mut().poll(&mut context) {
                return;
            }
            MockDriver::get().advance(Duration::from_micros(100));
        }
        panic!("mouse navigation test stalled");
    }

    // One test keeps the process-global mock clock and event channels isolated.
    #[test]
    fn mouse_navigation_preserves_layer_and_enter_types_normally() {
        run(async {
            let (mut data, mut behavior) = fixture();
            let positional = PositionalConfig::default();
            let keymap = KeyMap::new(&mut data, &mut behavior, &positional).await;
            let mut keyboard = Keyboard::new(&keymap);
            let mut auto_mouse = AutoMouseLayerRunner::new(&keymap);
            rmk::state::set_usb_state(UsbState::Configured);
            USB_REPORT_CHANNEL.clear();

            let scenario = async {
                Timer::after_millis(2).await;
                chord(&keymap, J, K, 1 << 3).await;
                chord(&keymap, K, J, 1 << 3).await;
                chord(&keymap, K, L, 1 << 4).await;
                chord(&keymap, L, K, 1 << 4).await;

                // Clicks still work after a navigation combo, without moving the ball.
                for (pos, buttons) in [(J, 1), (L, 2)] {
                    key(pos, true);
                    Timer::after_millis(70).await;
                    mouse_report(buttons);
                    assert_eq!(keymap.active_layer(), 4);
                    key(pos, false);
                    Timer::after_millis(2).await;
                    mouse_report(0);
                }

                typing(&keymap, ENTER, HidKeyCode::Enter).await;
                motion(&keymap).await;
                typing(&keymap, K, HidKeyCode::K).await;

                // The same physical chords must type letters in keyboard mode.
                for (first, second, letters) in [(J, K, [HidKeyCode::J, HidKeyCode::K]), (K, L, [HidKeyCode::K, HidKeyCode::L])] {
                    key(first, true);
                    key(second, true);
                    Timer::after_millis(70).await;
                    let mut seen = [false; 2];
                    while let Ok(report) = USB_REPORT_CHANNEL.try_receive() {
                        match report {
                            Report::KeyboardReport(report) => {
                                for (index, letter) in letters.iter().enumerate() {
                                    seen[index] |= report.keycodes.contains(&(*letter as u8));
                                }
                            }
                            _ => panic!("navigation fired while typing"),
                        }
                    }
                    assert_eq!(seen, [true; 2]);
                    assert_eq!(keymap.active_layer(), 0);
                    key(second, false);
                    key(first, false);
                    Timer::after_millis(2).await;
                    USB_REPORT_CHANNEL.clear();
                }
            };
            select(scenario, select(keyboard.run(), auto_mouse.run())).await;
        });
    }
}
