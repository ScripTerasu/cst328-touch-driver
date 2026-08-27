# `waveshare-esp32s3-touch-lcd-2p8`

This subproject (`examples/waveshare-esp32s3-touch-lcd-2p8`) started from the `esp-generate` template for the ESP32-S3 (ESP-HAL, Embassy, defmt, and `zed`). We replaced the stock "Hello world" loop with a CST328 touch demo that runs inside `esp-rtos` and logs touch points via `defmt`.

**This example targets the Waveshare ESP32-S3-Touch-LCD-2.8 board (CST328) specifically.** Its pin numbers are confirmed against that board's own schematic (`docs/ESP32-S3-Touch-LCD-2.8-schematic.pdf` at the repository root) — don't assume they carry over to any other board, including other Waveshare touch/LCD products.

This is the exact demo used to validate the driver on real hardware (2026-08-26) — flashed to an actual board, it correctly reported multi-touch coordinates and per-point pressure over a multi-hundred-report session with zero I2C errors. See the crate [README's Hardware notes](../../README.md#hardware-notes) for details.

It is **not** a workspace member of the crate at the repository root — it has its own `Cargo.lock` and toolchain pin, and is built from within this directory.

## What's inside?

- `Cargo.toml`: defines the `waveshare-esp32s3-touch-lcd-2p8` binary and pulls in `esp-hal`, `esp-rtos`, `embassy`, `defmt`, and the supporting ecosystem crates, plus `cst328` via a `path = "../.."` dependency.
- `src/bin/main.rs`: brings up the ESP32-S3 clocks, an async I²C bus dedicated to the touch controller, its RST pin, and its INT pin, spawns a `touch_task` that initializes the CST328 driver and reads `touches()` whenever INT toggles, and logs chip attributes and coordinates (including per-point pressure) via `defmt`.
- `.cargo/`, `.clippy.toml`, `rust-toolchain.toml`, and `build.rs`: boilerplate from `esp-generate` to pin the toolchain and lint rules.

## How to run it

1. Install the ESP toolchain (if you haven't yet):
   ```sh
   cargo install espup
   espup install
   ```
   This installs the `xtensa-esp` targets and helper tools such as `espflash` and `probe-rs`.

2. From **this directory** (`examples/waveshare-esp32s3-touch-lcd-2p8`), build or run the demo:
   ```sh
   cargo build --release
   cargo run --release
   ```
   The binary targets `#![no_std]` and links against `esp-rtos`, so task setup happens automatically.

3. Flash the binary to your board (replace `/dev/ttyUSB0` with the correct port):
   ```sh
   cargo espflash --release /dev/ttyUSB0
   ```
   Use the `defmt` feature built into `espflash` or pipe the ELF through `defmt-print` to decode the logs.

4. You can also monitor the serial port directly at 115200 bps using `minicom`, `picocom`, or `screen` if you need raw output.

## Integrating the CST328 driver

This template shows the minimal CST328 flow:

- `main()` configures I²C (SDA on GPIO1, SCL on GPIO3, 400 kHz, async — this is a bus dedicated to the touch controller on this board, **not** shared with the onboard IMU/RTC bus on GPIO10/11), drives RST on GPIO2 as a push-pull output idling high (the controller resets on a low pulse — see the driver [README](../../README.md#reset-pin)), configures INT on GPIO4 as an input with a pull-up, attaches RST with `.with_reset(rst)`, and spawns `touch_task` with both the driver and the interrupt pin. These pin assignments are confirmed against the official Waveshare schematic (`docs/ESP32-S3-Touch-LCD-2.8-schematic.pdf` at the repository root — net labels `TP_SDA`/`TP_SCL`/`TP_RST`/`TP_INT`) and cross-checked against a second, independent source: `github.com/zonfacter/ESP32-S3-Touch-LCD-2.8`'s `PIN_TOUCH_SDA`/`PIN_TOUCH_SCL`/`PIN_TOUCH_RST`/`PIN_TOUCH_INT` constants, which agree exactly.
- `touch_task` calls `touch_driver.init()` once at startup, which pulses RST and reads the chip attributes; on success it logs the model name (always `"CST328/CST3530"` — this driver can't tell the two chips apart, see the crate README) plus the full `ChipInfo` (panel resolution, firmware version, firmware CRC/checksum). It then loops on `touch_int.wait_for_any_edge().await` before each `touch_driver.touches()` call, instead of polling on a fixed interval — the chip only drives INT when it has a report ready, so the task (and the I²C bus) stay idle between touches. The CST328 datasheet says the interrupt edge is configurable (rising or falling) but doesn't say which one a given panel's firmware uses, so this waits on either edge rather than guessing. A failed `init()` or a per-read I²C error is logged with the actual `cst328::Error`/`esp_hal` error value, not just a generic message.
- Keep `cst328 = { path = "../..", features = ["defmt"] }` in `Cargo.toml` to reuse the driver crate from this repository; swap the `path` dependency for a version from crates.io in your own project.

Once you confirm the wiring for **your** board (I²C on GPIO1/3, RST on GPIO2, INT on GPIO4 — only correct as-is for this board), extend `touch_task` with your own gesture logic, or set an orientation/display mapping via `CST328::new(i2c, delay).with_reset(rst).with_config(config)`.

> Update this README whenever you change the build/flash workflow or the demo's behavior so it stays accurate for future flashes.
