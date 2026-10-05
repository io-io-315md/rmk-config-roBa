# roBa RMK firmware

RMK 0.9 firmware for the roBa split keyboard. This is a separate port of the
current ZMK layout so the RMK pointing stack can be tested without changing the
existing ZMK repository.

## Hardware roles

- `rmk-central.uf2`: right half, including the PMW3610 trackball
- `rmk-peripheral.uf2`: left half, including the rotary encoder
- Controller: Seeed XIAO nRF52840 on both halves
- Split transport: Bluetooth LE
- Host transport: Bluetooth LE 1M PHY for Windows adapter compatibility

## Preserved behavior

- Eight keymap layers, combos, Bluetooth profile keys and the encoder scroll map
- Tap Zen/Han with hold for layer 1 on the left thumb key
- Tap Tab with hold for layer 2 on the next left thumb key
- Tap Space on the outer left thumb key
- Tap Backspace with a 100 ms hold for Left Shift; quick double-tap hold repeats Backspace
- Tap Delete, or Escape while Shift is held, on the lower-right key
- Trackball motion activates the J/L mouse-button layer
- In mouse mode, J+K sends mouse button 4 (Back), and K+L sends mouse button 5 (Forward)
- Mouse combos use a 50 ms window and keep mouse mode; in keyboard mode J/K/L type normally
- Enter always sends Enter and returns to keyboard mode
- J and L keep mouse mode; other keys immediately return to keyboard mode
- PMW3610 reports at 125 Hz to avoid flooding the BLE event path
- The host connection uses 1M PHY; the split link between halves remains 2M PHY

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

The current firmware is ZMK, so use the included ZMK settings-reset image once
on both halves before the first RMK flash. The same reset file is used for both
XIAO nRF52840 controllers:

1. Put the left half into its UF2 bootloader and copy
   `settings_reset-seeeduino_xiao_ble-zmk.uf2` to it.
2. Put the left half into the bootloader again and copy `rmk-peripheral.uf2`.
3. Put the right half into its UF2 bootloader and copy the same settings-reset
   UF2 to it.
4. Put the right half into the bootloader again and copy `rmk-central.uf2`.
5. Remove the old `roBa` pairing from the host and pair `roBa RMK`.

The reset image is only needed for this first migration from ZMK. Do not flash
it again for ordinary RMK firmware updates because it clears saved settings and
Bluetooth bonds.
