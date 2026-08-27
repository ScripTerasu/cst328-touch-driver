# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-26

Published as **`cst328-touch-driver`** on crates.io, not `cst328` — that name was already taken by an unrelated, independent driver ([`cmumford/cst328`](https://github.com/cmumford/cst328)) for the same chip, discovered while preparing this release. The importable crate name is still `cst328` (`[lib].name` in `Cargo.toml`); depend on it with `cst328 = { package = "cst328-touch-driver", version = "0.1" }` — see [README.md#installation](README.md#installation).

Initial release: `no_std` CST328/CST3530 driver (async `embedded-hal-async` by default, blocking `embedded-hal` counterpart via the `blocking` feature), shared register/error/mode/info types, optional `defmt` support, and coordinate transform (`TouchConfig`/`Orientation`/`DisplayMapping`).

Ported from Waveshare's official [`esp_lcd_touch_cst328`](https://github.com/waveshareteam/Waveshare-ESP32-components/blob/main/display/touch/esp_lcd_touch_cst328/esp_lcd_touch_cst328.c) component and ESPHome's [`cst328`](https://github.com/esphome/esphome/blob/dev/esphome/components/cst328/touchscreen/cst328_touchscreen.cpp) component, **not** SensorLib — SensorLib has no `TouchDrvCST328`; its `CST328_SLAVE_ADDRESS` alias resolves to `TouchDrvCST3530`, which speaks a different, incompatible protocol. See [README.md](README.md#why-this-isnt-ported-from-sensorlib) for the full rationale.

Cross-checked against Hynitron's official CST328 (V2.2) and CST3530 (V1.0) datasheets and the Waveshare ESP32-S3-Touch-LCD-2.8 board schematic — all kept in `docs/` alongside verbatim copies of the Waveshare/ESPHome/SensorLib source this crate was ported from (or deliberately wasn't), with license attribution in `docs/README.md`. The datasheet's register appendix confirmed the ported register map byte-for-byte and added three registers neither reference driver reads/writes (`REG_DEBUG_WRITE_MODE`/`0xD10B`, `REG_DEBUG_CALIBRATION_MODE`/`0xD10C`, and `REG_FW_CHECKSUM`/`0xD20C`, the last now populating `ChipInfo::fw_checksum`); `RunMode` gained `DebugWrite` and `DebugCalibration` variants for the first two. `decode_touch_report()` additionally validates the fixed `0xAB` frame marker the datasheet documents at report offset 6, which neither reference driver checks itself.

**Validated on real hardware** (2026-08-26): flashed the included `examples/waveshare-esp32s3-touch-lcd-2p8` demo to an actual Waveshare ESP32-S3-Touch-LCD-2.8 board. `reset()`, `get_attribute()` (including the `0xCACA` firmware-CRC check), and `touches()` (including the `0xAB` frame-marker validation, real per-point pressure, and two simultaneous touch points) all worked correctly over a multi-hundred-report session with zero I2C errors or panics. The example's I2C/RST/INT pins were confirmed against the official schematic and a second independent source. See [README.md#hardware-notes](README.md#hardware-notes) for what's still unverified (most `RunMode` variants beyond `Normal`/`DebugInfo`, and CST3530 specifically) and for a documented, deliberate divergence from Waveshare's driver (a two-step touch read this crate doesn't replicate).

[Unreleased]: https://github.com/ScripTerasu/cst328-touch-driver/compare/0.1.0...HEAD
[0.1.0]: https://github.com/ScripTerasu/cst328-touch-driver/releases/tag/0.1.0
