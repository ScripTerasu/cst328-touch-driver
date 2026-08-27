# docs/

Reference material for the CST328/CST3530 protocol this crate implements.

## `reference/`

Verbatim copies of the source this driver was ported from (and one source it deliberately *wasn't*), kept here so the byte-level protocol details in `src/registers.rs`/`src/protocol.rs`/`src/driver.rs` can be checked against their origin without re-fetching from GitHub. Each subfolder is one upstream project; nothing here has been modified from the original.

| Folder | Files | License | Source |
| --- | --- | --- | --- |
| `reference/waveshare/` | `esp_lcd_touch_cst328.c`, `.h` | Apache-2.0 (Espressif Systems / Waveshare) | [`waveshareteam/Waveshare-ESP32-components`](https://github.com/waveshareteam/Waveshare-ESP32-components/tree/main/display/touch/esp_lcd_touch_cst328) — Waveshare's official ESP-IDF component, written for the exact chip on the [ESP32-S3-Touch-LCD-2.8](https://www.waveshare.com/esp32-s3-touch-lcd-2.8.htm) board this crate targets. |
| `reference/esphome/` | `cst328_touchscreen.cpp`, `.h` | MIT (ESPHome project) | [`esphome/esphome`](https://github.com/esphome/esphome/tree/dev/esphome/components/cst328/touchscreen) — an independent, second implementation of the same register protocol; cross-corroborates the Waveshare driver byte-for-byte (see `src/protocol.rs` doc comments for where they agree). |
| `reference/sensorlib/` | `TouchDrvCST3530.cpp`, `.hpp` | MIT (Lewis He / SensorLib) | [`lewisxhe/SensorLib`](https://github.com/lewisxhe/SensorLib/tree/master/src/touch) — **not** the protocol this crate implements. Kept here specifically so the difference is checkable: SensorLib has no `TouchDrvCST328`; its `CST328_SLAVE_ADDRESS` I2C-address alias resolves to this driver, which speaks an unrelated 4-byte command-wrapper protocol. See the main [README](../README.md#why-this-isnt-ported-from-sensorlib) for the full explanation of why this crate ports the Waveshare/ESPHome protocol instead. |

### Mapping to this crate

| Reference detail | This crate |
| --- | --- |
| Register map (`0xD000`, `0xD005`, `0xD101`-`0xD120`, `0xD1F4`-`0xD1FC`, `0xD204`, `0xD208`) | [`src/registers.rs`](../src/registers.rs) |
| Reset timing (50 ms / 5 ms / 300 ms), attribute-read sequence | [`src/driver.rs`](../src/driver.rs) (`reset()`, `get_attribute()`) |
| Touch report byte layout, `0xCACA` firmware-CRC check | [`src/protocol.rs`](../src/protocol.rs) |
| Run-mode registers, including the ones neither reference driver actually exercises | [`src/mode.rs`](../src/mode.rs) |

No datasheet PDF is included here (unlike the sister [`cst9217`](https://github.com/ScripTerasu/cst92xx-touch-driver) crate's `docs/CST9217.pdf`) — no CST328/CST3530 datasheet was available at the time this driver was written. If you have one, adding it here alongside a note on which register-map details it confirms or corrects would close that gap.
