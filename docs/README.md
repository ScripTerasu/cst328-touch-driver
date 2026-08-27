# docs/

Reference material for the CST328/CST3530 protocol this crate implements.

## Datasheets and schematic

| File | Source | Notes |
| --- | --- | --- |
| [`CST328_Datasheet_zh_V2.2.pdf`](CST328_Datasheet_zh_V2.2.pdf) | Hynitron Microelectronics, via [Xinyuan-LilyGO/T-Display-S3](https://github.com/Xinyuan-LilyGO/T-Display-S3/blob/main/datasheet/CST328%E6%95%B0%E6%8D%AE%E6%89%8B%E5%86%8CV2.2.pdf) | Chinese-language official datasheet. Section 12 ("寄存器附录", register appendix) confirms the register map this crate ports byte-for-byte, and adds `0xD10B`/`0xD10C`/`0xD20C` — registers neither reference driver in `reference/` happens to use. |
| [`CST3530_Datasheet_V1.0.pdf`](CST3530_Datasheet_V1.0.pdf) | Hynitron Technology, via [osptek.com](https://admin.osptek.com/uploads/CST_3530_V1_0_adb72690f0.pdf) | Confirms the same default I2C address (8-bit `0x34`/`0x35`, 7-bit `0x1A`) as CST328, but has **no register appendix at all** — this crate's assumption that CST3530 speaks the same register protocol as CST328 (inherited from SensorLib's address-alias precedent) remains unconfirmed by this document. It also describes a meaningfully different chip: 30 channels vs. CST328's 28, up to 10 real touch points vs. 5, `VDDA` up to 5.5V vs. 3.6V, a different QFN41 pinout. If [`MAX_FINGER_NUM`](../src/registers.rs)/`TOUCH_DATA_SIZE` (sized for 5 points) turn out not to fit genuine CST3530 silicon, this is the most likely reason. |
| [`ESP32-S3-Touch-LCD-2.8-schematic.pdf`](ESP32-S3-Touch-LCD-2.8-schematic.pdf) | Waveshare, [`files.waveshare.com/wiki/ESP32-S3-Touch-LCD-2.8/...`](https://files.waveshare.com/wiki/ESP32-S3-Touch-LCD-2.8/ESP32-S3-Touch-LCD-2.8.pdf) | Board schematic for the [ESP32-S3-Touch-LCD-2.8](https://www.waveshare.com/esp32-s3-touch-lcd-2.8.htm) module `examples/waveshare-esp32s3-touch-lcd-2p8` targets. Confirms the touch controller's I2C SDA/SCL, RST, and INT pins (net labels `TP_SDA`/`TP_SCL`/`TP_RST`/`TP_INT`), cross-checked against a second, independent source — see the example's README. |

## `reference/`

Verbatim copies of the source this driver was ported from (and one source it deliberately *wasn't*), kept here so the byte-level protocol details in `src/registers.rs`/`src/protocol.rs`/`src/driver.rs` can be checked against their origin without re-fetching from GitHub. Each subfolder is one upstream project; nothing here has been modified from the original.

| Folder | Files | License | Source |
| --- | --- | --- | --- |
| `reference/waveshare/` | `esp_lcd_touch_cst328.c`, `.h` | Apache-2.0 (Espressif Systems / Waveshare) | [`waveshareteam/Waveshare-ESP32-components`](https://github.com/waveshareteam/Waveshare-ESP32-components/tree/main/display/touch/esp_lcd_touch_cst328) — Waveshare's official ESP-IDF component, written for the exact chip on the [ESP32-S3-Touch-LCD-2.8](https://www.waveshare.com/esp32-s3-touch-lcd-2.8.htm) board this crate targets. |
| `reference/esphome/` | `cst328_touchscreen.cpp`, `.h` | MIT (ESPHome project) | [`esphome/esphome`](https://github.com/esphome/esphome/tree/dev/esphome/components/cst328/touchscreen) — an independent, second implementation of the same register protocol; cross-corroborates the Waveshare driver byte-for-byte (see `src/protocol.rs` doc comments for where they agree). |
| `reference/sensorlib/` | `TouchDrvCST3530.cpp`, `.hpp` | MIT (Lewis He / SensorLib) | [`lewisxhe/SensorLib`](https://github.com/lewisxhe/SensorLib/tree/master/src/touch) — **not** the protocol this crate implements. Kept here specifically so the difference is checkable: SensorLib has no `TouchDrvCST328`; its `CST328_SLAVE_ADDRESS` I2C-address alias resolves to this driver, which speaks an unrelated 4-byte command-wrapper protocol. See the main [README](../README.md#why-this-isnt-ported-from-sensorlib) for the full explanation of why this crate ports the Waveshare/ESPHome protocol instead. |

### Mapping to this crate

| Reference detail | Source | This crate |
| --- | --- | --- |
| Register map (`0xD000`-`0xD01A`, `0xD101`-`0xD120`, `0xD1F8`-`0xD20C`) | Waveshare/ESPHome source, confirmed by the CST328 datasheet §12 | [`src/registers.rs`](../src/registers.rs) |
| Reset timing (50 ms / 5 ms / 300 ms), attribute-read sequence | ESPHome source (`CST328_BEFORE_RESET_TIMEOUT`/`CST328_TRANSITION_TIMEOUT` comments cite "from datasheet") | [`src/driver.rs`](../src/driver.rs) (`reset()`, `get_attribute()`) |
| Touch report byte layout, `0xCACA` firmware-CRC check, `0xAB` frame marker at offset 6 | Waveshare/ESPHome source; offset-6 marker and the `0xD000` touch-info table independently confirmed by the CST328 datasheet §12 | [`src/protocol.rs`](../src/protocol.rs) |
| Run-mode registers — `0xD101`/`0xD109` exercised by the reference drivers; `0xD102`/`0xD104`/`0xD105`/`0xD108`/`0xD10A`/`0xD10D`/`0xD119` datasheet-confirmed but not exercised by either driver; `0xD10B`/`0xD10C` datasheet-only (absent from both reference drivers' source); `0xD120` reference-driver-only (absent from the datasheet) | CST328 datasheet §12 mode-command table, cross-checked against Waveshare's declared-but-unused register constants | [`src/mode.rs`](../src/mode.rs) |
| Touch I2C pinout for the Waveshare ESP32-S3-Touch-LCD-2.8 (SDA=GPIO1, SCL=GPIO3, RST=GPIO2, INT=GPIO4) | Waveshare's official schematic PDF, cross-checked against `github.com/zonfacter/ESP32-S3-Touch-LCD-2.8` | [`examples/waveshare-esp32s3-touch-lcd-2p8`](../examples/waveshare-esp32s3-touch-lcd-2p8) |
