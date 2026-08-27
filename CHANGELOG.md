# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-26

Initial release: `no_std` CST328/CST3530 driver (async `embedded-hal-async` by default, blocking `embedded-hal` counterpart via the `blocking` feature), shared register/error/mode/info types, optional `defmt` support, and coordinate transform (`TouchConfig`/`Orientation`/`DisplayMapping`).

Ported from Waveshare's official [`esp_lcd_touch_cst328`](https://github.com/waveshareteam/Waveshare-ESP32-components/blob/main/display/touch/esp_lcd_touch_cst328/esp_lcd_touch_cst328.c) component and ESPHome's [`cst328`](https://github.com/esphome/esphome/blob/dev/esphome/components/cst328/touchscreen/cst328_touchscreen.cpp) component, **not** SensorLib — SensorLib has no `TouchDrvCST328`; its `CST328_SLAVE_ADDRESS` alias resolves to `TouchDrvCST3530`, which speaks a different, incompatible protocol. See [README.md](README.md#why-this-isnt-ported-from-sensorlib) for the full rationale.

Not yet validated against real hardware — see [README.md#hardware-notes](README.md#hardware-notes).

[Unreleased]: https://github.com/ScripTerasu/cst328-touch-driver/compare/0.1.0...HEAD
[0.1.0]: https://github.com/ScripTerasu/cst328-touch-driver/releases/tag/0.1.0
