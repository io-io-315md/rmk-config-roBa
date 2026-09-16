# roBa RMK firmware

RMK 0.9 firmware for the roBa split keyboard. This is a separate port of the
current ZMK layout so the RMK pointing stack can be tested without changing the
existing ZMK repository.

## Hardware roles

- `rmk-central.uf2`: right half, including the PMW3610 trackball
- `rmk-peripheral.uf2`: left half, including the rotary encoder
- Controller: Seeed XIAO nRF52840 on both halves
- Split transport: Bluetooth LE

## Preserved behavior

- Eight keymap layers, combos, Bluetooth profile keys and the encoder scroll map
- Tap Zen/Han with hold for layer 1 on the left thumb key
- Tap Tab with hold for layer 2 on the next left thumb key
- Tap Space on the outer left thumb key
- Tap Backspace with a 100 ms hold for Left Shift; quick double-tap hold repeats Backspace
- Tap Delete, or Escape while Shift is held, on the lower-right key
- Trackball motion activates the J/L mouse-button layer
- Motion distance above 10 also enables Enter as mouse button 4 for one press
- J and L keep mouse mode; other keys immediately return to keyboard mode
- PMW3610 reports at 125 Hz to avoid flooding the BLE event path

## Build

GitHub Actions builds both UF2 files on every push to `main`. Download the
`roBa-RMK-firmware` artifact from the latest successful workflow run.

Local builds require Rust 1.96, `cargo-make`, `flip-link`, ARM GCC,
`cargo-binutils`, and `cargo-hex-to-uf2`:

```sh
cargo build --release
cargo make uf2 --release
```

## First RMK flash

Flash both halves when moving from ZMK to RMK. Remove the old keyboard pairing
from the host, then pair the device named `roBa RMK`. The old ZMK settings in
flash are not used as RMK settings.

