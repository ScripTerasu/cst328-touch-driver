# CST328/CST3530 Touch Controller Driver

[![CI](https://github.com/ScripTerasu/cst328-touch-driver/actions/workflows/ci.yml/badge.svg)](https://github.com/ScripTerasu/cst328-touch-driver/actions/workflows/ci.yml)

`cst328` is a `no_std` driver for the CST328/CST3530 family of capacitive touch controllers, ported from Waveshare's official [`esp_lcd_touch_cst328`][waveshare-driver] component and ESPHome's [`cst328`][esphome-driver] component to idiomatic `embedded-hal`. It exposes one `CST328` type backed by either async or blocking I²C, shared register/type modules, and the `RunMode` enum so you can plug it into any embedded project.

See [CHANGELOG.md](CHANGELOG.md) for release history.

## Why this isn't ported from SensorLib

Sister project [`cst92xx`](https://github.com/ScripTerasu/cst92xx-touch-driver) was ported from SensorLib's `TouchDrvCST92xx`. This crate isn't, on purpose: **SensorLib has no `TouchDrvCST328`.** Its `CST328_SLAVE_ADDRESS` constant is only an I2C-address alias (`0x1A`) that its generic dispatcher resolves to `TouchDrvCST3530` — a driver that speaks a completely different protocol (a 4-byte command wrapper: `0xD0070000` to read touches, `0xD00002AB` to ack, etc.), not the 16-bit register map (`0xD000`, `0xD005`, `0xD1F8`, `0xD1FC`, `0xD204`, `0xD208`, ...) documented for CST328.

This driver ports the register-based protocol instead, cross-verified against two independent, actively-maintained, open-source drivers written specifically for this chip on the [Waveshare ESP32-S3-Touch-LCD-2.8](https://www.waveshare.com/esp32-s3-touch-lcd-2.8.htm) board, and confirmed against Hynitron's own official CST328 datasheet:

- Waveshare's own [`esp_lcd_touch_cst328`][waveshare-driver] ESP-IDF component.
- ESPHome's [`cst328`][esphome-driver] component.
- Hynitron's official CST328 datasheet (`docs/CST328_Datasheet_zh_V2.2.pdf`), section 12 — its register appendix confirms this map byte-for-byte and adds a few registers neither reference driver happens to use (see [Constants](#constants) and [`mode.rs`](src/mode.rs)).

All three agree on the register map, the reset timing, and the touch-report byte layout (down to the same "point 0 is a 7-byte stride, points 1-4 are 5-byte strides" quirk), which is why this crate follows them rather than SensorLib's `TouchDrvCST3530`. Hynitron's separate CST3530 datasheet confirms the same default I2C address but has **no register appendix at all** — see [Chip identification](#chip-identification-chipinfochip_id) below.

## Feature flags

| Feature | Default | Effect |
| --- | --- | --- |
| `async` | yes | `CST328::new(i2c, delay)` over `embedded_hal_async::i2c::I2c` + `embedded_hal_async::delay::DelayNs`. |
| `blocking` | no | `CST328::new(i2c, delay)` over `embedded_hal::i2c::I2c` + `embedded_hal::delay::DelayNs`. |
| `defmt` | no | Derives `defmt::Format` on `Point`, `ChipInfo`, `TouchConfig`, `RunMode`, and `Error` for logging. |

`async` and `blocking` both export a `CST328` type at the crate root and are **mutually exclusive** — enabling both (e.g. `cargo build --all-features`) fails to compile with a clear error. To use the blocking driver:

```toml
[dependencies]
cst328 = { version = "0.1", default-features = false, features = ["blocking"] }
```

## Usage example (async)

```rust
use cst328::{CST328, RunMode};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

let mut driver = CST328::new(i2c, delay); // any embedded_hal_async::delay::DelayNs works,
                                           // e.g. embassy_time::Delay if you already use embassy
// Optional: attach a real RST line and/or an orientation/display mapping.
// let mut driver = CST328::new(i2c, delay).with_reset(rst_pin).with_config(config);

// 1. Initialize the controller (reset + attribute read)
driver.init().await?;

// 2. Fetch all touch points (up to 5 entries)
let touches = driver.touches().await?;
for point in touches.iter().flatten() {
    // handle point (point.x, point.y, point.track_id, point.area == pressure)
}
```

## Usage example (blocking)

```rust
use cst328::{CST328, RunMode};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

fn scan<I2C, D, E>(i2c: I2C, delay: D) -> Result<(), cst328::Error<E>>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    let mut driver = CST328::new(i2c, delay);

    driver.init()?;

    let touches = driver.touches()?;
    for point in touches.iter().flatten() {
        // handle point
    }

    Ok(())
}
```

## API overview

| Item | Description |
| --- | --- |
| `CST328<I2C, Delay>` (feature `async`) | Async driver over `embedded-hal-async::i2c::I2c` + `embedded-hal-async::delay::DelayNs`. Provides `init`, `touches`, `set_mode`, `chip_info`, `model_name`, `with_reset`, and `with_config`. |
| `CST328<I2C, Delay>` (feature `blocking`) | Sync counterpart over `embedded_hal::i2c::I2c` + `embedded_hal::delay::DelayNs`. Mirrors the async API so business logic reads the same between runtimes. |
| `ChipInfo` | Chip metadata discovered by `init()`/`get_attribute()` — chip/project ID, panel resolution, firmware version, firmware CRC. Read-only; fetch it with `driver.chip_info()`. |
| `Point` | Touch descriptor returned by `touches()`, already passed through `TouchConfig::transform()`. Includes `track_id`, `(x, y)`, and `area` — a real pressure/weight value on this chip family (unlike CST92xx, which always reports `0`). |
| `TouchConfig` / `Orientation` / `DisplayMapping` | Optional coordinate transform (axis swap, mirroring, scaling to a target display resolution) applied to every `Point` from `touches()`. Set it via `driver.with_config(...)`; see `TouchConfig::with_target_resolution`. |
| `RunMode` | Enum describing every controller mode (normal, debug/info, factory test, etc.) for `set_mode`. Only `Normal` and `DebugInfo` are exercised by either reference driver; most other variants are still datasheet-confirmed (Hynitron's official register appendix documents them) even though neither reference driver writes them — `DebugWrite`/`DebugCalibration` are datasheet-only, and `Factory2` is the one variant with no datasheet backing at all. See the per-variant docs before relying on them. |
| `NoResetPin` | Default `RST` type when no hardware reset pin is attached (via `.with_reset(pin)`). `reset()` still runs its settle delays, just without toggling anything. |
| `Error<E>` | Driver error type, generic over the I²C error type `E`. |

- `touches()` reads the `REG_READ` report, validates the fixed `0xAB` frame marker Hynitron's datasheet documents at offset 6 (rejecting the report if it doesn't match), decodes up to 5 points, and applies `TouchConfig::transform()` before returning. It acknowledges **every** read regardless of content (clearing `REG_FINGER_NUM` and re-arming the `0xAB` sync byte at `REG_READ`), matching both reference drivers — this intentionally does not replicate the CST92xx driver's "skip the ack on an all-zero buffer" behavior, since neither reference implementation here does that either.
- `set_mode()` writes a zero-length payload to the target mode's work-mode register. Unlike CST92xx, no confirmation/status-echo register is known for this protocol, so there's no handshake or retry loop.
- Use `driver.model_name()` for a human-readable chip name (always `"CST328/CST3530"` — see [`ChipInfo::chip_id`] below for why), or `driver.chip_info()` for the full `ChipInfo`.

## Reset pin

Both drivers default to `NoResetPin`, a no-op `OutputPin` — `reset()` still waits out the settle delays but never drives a real pin, which is fine if you only rely on power-on reset. To drive a real `RST` line:

```rust
let mut driver = CST328::new(i2c, delay).with_reset(rst_pin);
```

Timing matches ESPHome's `cst328` component (the clearer of the two reference drivers on this point, and the one with an explicit datasheet-timing comment): ensure the pin starts released (high), wait 50 ms, assert low for 5 ms, release high, then wait 300 ms before touching the chip over I2C. Waveshare's own driver uses a shorter, less-documented 10 ms/10 ms pulse that never explicitly re-releases the pin at the end of its `reset()` function — this driver follows ESPHome's version instead since it's unambiguous and datasheet-referenced.

## Chip identification (`ChipInfo::chip_id`)

Neither reference driver validates `REG_CHIP_TYPE`'s chip-ID field against a known table — both read it only to log it, and Hynitron's official CST328 datasheet documents the field (`IC_TYPE`) without listing what values it actually takes. That means this driver **cannot tell CST328 and CST3530 apart** from any register value currently documented anywhere; `model_name()` always returns `"CST328/CST3530"`, and `ChipInfo::chip_id`/`ChipInfo::project_id` are exposed as raw, informational values only. The only value-based validation `get_attribute()` performs is the firmware CRC at `REG_CHECK_CODE` (`0xD1FC`, bytes 2-3), which both reference drivers — and the datasheet — require to equal `0xCACA`.

Hynitron's separate CST3530 datasheet (`docs/CST3530_Datasheet_V1.0.pdf`) confirms the same default I2C address as CST328, but has **no register appendix at all** — this crate's CST3530 support remains an assumption, inherited from SensorLib's address-alias precedent, that it speaks the same register protocol as CST328. The CST3530 datasheet also describes a meaningfully bigger chip (30 channels vs. 28, up to 10 real touch points vs. 5, a wider `VDDA` range, a different QFN41 pinout), so if this driver's `MAX_FINGER_NUM`/`TOUCH_DATA_SIZE` (both sized for CST328's 5-point protocol) turn out not to fit genuine CST3530 silicon, that mismatch is the most likely reason.

## Constants

The full register map and protocol constants live in [`registers.rs`](src/registers.rs) (see docs.rs for the complete, documented list). The most commonly useful ones:

| Name | Description |
| --- | --- |
| `CST328_SLAVE_ADDRESS` | Fixed I²C address (0x1A). |
| `MAX_FINGER_NUM` | Maximum simultaneous contacts this driver decodes (5). |
| `CST328_SYNC_BYTE` | Ack/sync byte (`0xAB`) written to `REG_READ` after every touch report and once during `get_attribute()` — also the value `touches()` expects to read back at offset 6 of every report, per the datasheet's fixed frame marker. |
| `REG_FW_CHECKSUM` | Firmware checksum register (`0xD20C`), datasheet-documented but not validated against anything — populates `ChipInfo::fw_checksum`, informational only. |

## Errors

```rust
pub enum Error<E> {
    InvalidCheckCode, // get_attribute() read a firmware CRC that didn't match the expected 0xCACA
    I2C(E),            // pass-through I²C error
}
```

## Optional `defmt` feature

Enable the `defmt` feature if you want the helper types and log statements to derive `defmt::Format`:

```toml
[dependencies]
cst328 = { version = "0.1", features = ["defmt"] }
```

## Development

```sh
cargo fmt
cargo test                                           # default (async) feature
cargo test --no-default-features --features blocking
cargo clippy --all-targets -- -D warnings
cargo clippy --no-default-features --features blocking --all-targets -- -D warnings
```

Run the usual tooling before deploying to hardware. `cargo build --all-features` is expected to fail — `async` and `blocking` are mutually exclusive (see [Feature flags](#feature-flags)).

## Hardware notes

This driver targets the CST328/CST3530 controller as wired on the [Waveshare ESP32-S3-Touch-LCD-2.8](https://www.waveshare.com/esp32-s3-touch-lcd-2.8.htm) module, at I²C address `0x1A`. It hasn't yet been validated against real hardware after being written — the byte layouts and register map are cross-corroborated between two independent reference drivers *and* Hynitron's own official CST328 datasheet (see "Why this isn't ported from SensorLib" above), but this driver's own Rust port of them hasn't been flashed and touched yet. Treat it as a strong first draft, not a proven-on-hardware release, until that validation happens.

A few things worth re-checking once you do have hardware to test against:

- `RunMode` variants the reference drivers don't exercise, even the datasheet-confirmed ones — a documented command byte isn't the same as this driver's exact write sequence being verified against silicon. See the per-variant docs in [`mode.rs`](src/mode.rs).
- The reset timing (50 ms / 5 ms / 300 ms) — ported from ESPHome's datasheet-referenced comments, not independently re-measured. (Hynitron's own datasheet gives different, smaller numbers for its power-on/reset timing — `TPOR`≈200 ms, `TRON`≈200 ms, `TRST`≈0.1 ms typical — which this driver's timing comfortably exceeds, but the two aren't the same numbers.)
- Whether `touches()`'s unconditional ack (versus CST92xx's skip-on-all-zero) is actually necessary/correct for every report the real chip sends.
- CST3530 support specifically — see [Chip identification](#chip-identification-chipinfochip_id) above; it's a bigger, meaningfully different chip with no independently published register map.

## References

Verbatim copies of everything below (plus both datasheets and the board schematic) are kept in [`docs/`](docs) (see [`docs/README.md`](docs/README.md) for licenses and a table mapping each source to the crate file it informed), so the protocol details can be checked without re-fetching from GitHub:

- [`esp_lcd_touch_cst328.c`][waveshare-driver] / [`esp_lcd_touch_cst328.h`](https://github.com/waveshareteam/Waveshare-ESP32-components/blob/main/display/touch/esp_lcd_touch_cst328/include/esp_lcd_touch_cst328.h) — Waveshare's official ESP-IDF component
- [`cst328_touchscreen.cpp`][esphome-driver] / [`cst328_touchscreen.h`](https://github.com/esphome/esphome/blob/dev/esphome/components/cst328/touchscreen/cst328_touchscreen.h) — ESPHome's component
- [Hynitron CST328 datasheet](docs/CST328_Datasheet_zh_V2.2.pdf) (Chinese, V2.2) — official register map, electrical characteristics, reset timing
- [Hynitron CST3530 datasheet](docs/CST3530_Datasheet_V1.0.pdf) (Chinese, V1.0) — confirms the shared default I2C address; no register appendix
- [SensorLib `TouchDrvCST3530.cpp`](https://github.com/lewisxhe/SensorLib/blob/master/src/touch/TouchDrvCST3530.cpp) / [`.hpp`](https://github.com/lewisxhe/SensorLib/blob/master/src/touch/TouchDrvCST3530.hpp) by Lewis He — the *different* protocol this crate deliberately does not implement; see "Why this isn't ported from SensorLib" above
- [Waveshare ESP32-S3-Touch-LCD-2.8 schematic](docs/ESP32-S3-Touch-LCD-2.8-schematic.pdf) — used by `examples/waveshare-esp32s3-touch-lcd-2p8`

[waveshare-driver]: https://github.com/waveshareteam/Waveshare-ESP32-components/blob/main/display/touch/esp_lcd_touch_cst328/esp_lcd_touch_cst328.c
[esphome-driver]: https://github.com/esphome/esphome/blob/dev/esphome/components/cst328/touchscreen/cst328_touchscreen.cpp
