# ZMK-to-RMK migration reset

`settings_reset-seeeduino_xiao_ble-zmk.uf2` is the common ZMK settings-reset
firmware for both Seeed XIAO nRF52840 halves.

It was copied from the successful ZMK firmware artifact built from:

- Repository: `io-io-315md/zmk-config-roBa`
- Commit: `df7a4c635f2199af61ede82d1cdab499f58570dc`
- Workflow run: `34808984746`
- SHA-256: `4F0CC7DB53BBC11F19510589F0DE14A19FD36178F0BC7CBF84E508776DFC23FF`

Flash it once to each half before installing RMK for the first time. It is not
needed for later RMK updates.

