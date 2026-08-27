# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `docs/CST328_Datasheet_zh_V2.2.pdf` and `docs/CST3530_Datasheet_V1.0.pdf`, Hynitron's official datasheets, plus `docs/ESP32-S3-Touch-LCD-2.8-schematic.pdf` (Waveshare's board schematic). The CST328 datasheet's register appendix (section 12) confirms the register map this crate ported from Waveshare/ESPHome byte-for-byte, and documents three registers neither reference driver reads/writes: `REG_DEBUG_WRITE_MODE` (`0xD10B`), `REG_DEBUG_CALIBRATION_MODE` (`0xD10C`), and `REG_FW_CHECKSUM` (`0xD20C`, now populating a new `ChipInfo::fw_checksum` field). `RunMode` gained `DebugWrite` and `DebugCalibration` variants for the first two.
- `docs/reference/`: verbatim copies of the Waveshare, ESPHome, and SensorLib source this crate was ported from (or deliberately wasn't), with license attribution.
- `examples/waveshare-esp32s3-touch-lcd-2p8`, an ESP32-S3 + Embassy demo for the board this crate targets, with I2C/RST/INT pins confirmed against the official schematic and a second independent source.
- `decode_touch_report()` now validates the fixed `0xAB` frame marker Hynitron's datasheet documents at report offset 6, rejecting a report if it doesn't match — the same role SensorLib's own `0xAB` ack byte plays for the CST92xx protocol, but not something either CST328 reference driver checks.
- `CHANGELOG.md` (this file).

### Changed

- `RunMode` variant docs (`Reset`, `DebugRecalibration`, `DeepSleep`, `DebugPoint`, `DebugRawData`, `DebugDiff`, `Factory`) upgraded from "unverified, register-naming-convention only" to "datasheet-confirmed, just not exercised by either reference driver" now that Hynitron's datasheet documents each one's actual command. `Factory2` (`0xD120`) is now flagged as the one variant with *no* datasheet backing at all — it's absent from the datasheet's register appendix entirely, unlike everything else in `registers.rs`.

## [0.1.0] - 2026-08-26

Initial release: `no_std` CST328/CST3530 driver (async `embedded-hal-async` by default, blocking `embedded-hal` counterpart via the `blocking` feature), shared register/error/mode/info types, optional `defmt` support, and coordinate transform (`TouchConfig`/`Orientation`/`DisplayMapping`).

Ported from Waveshare's official [`esp_lcd_touch_cst328`](https://github.com/waveshareteam/Waveshare-ESP32-components/blob/main/display/touch/esp_lcd_touch_cst328/esp_lcd_touch_cst328.c) component and ESPHome's [`cst328`](https://github.com/esphome/esphome/blob/dev/esphome/components/cst328/touchscreen/cst328_touchscreen.cpp) component, **not** SensorLib — SensorLib has no `TouchDrvCST328`; its `CST328_SLAVE_ADDRESS` alias resolves to `TouchDrvCST3530`, which speaks a different, incompatible protocol. See [README.md](README.md#why-this-isnt-ported-from-sensorlib) for the full rationale.

Not yet validated against real hardware — see [README.md#hardware-notes](README.md#hardware-notes).

[Unreleased]: https://github.com/ScripTerasu/cst328-touch-driver/compare/0.1.0...HEAD
[0.1.0]: https://github.com/ScripTerasu/cst328-touch-driver/releases/tag/0.1.0
