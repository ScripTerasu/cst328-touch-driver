#![no_std]

//! A `no_std` driver for the CST328/CST3530 capacitive touch controllers,
//! ported from Waveshare's official [`esp_lcd_touch_cst328`][waveshare]
//! component and ESPHome's [`cst328`][esphome] component to idiomatic
//! `embedded-hal`.
//!
//! **Not ported from SensorLib.** SensorLib has no `TouchDrvCST328` — its
//! `CST328_SLAVE_ADDRESS` constant is only an I2C-address alias resolved to
//! `TouchDrvCST3530`, which speaks a different, incompatible protocol (a
//! 4-byte command wrapper, rather than the 16-bit register reads/writes this
//! crate implements). This crate ports the "classic" register-based
//! protocol instead, since that's what's confirmed against real hardware by
//! two independent, actively-maintained open-source drivers written
//! specifically for this chip.
//!
//! The crate exposes one `CST328` type backed by either `embedded-hal-async`
//! or blocking `embedded-hal` I²C, selected via Cargo features:
//!
//! | Feature | Default | Effect |
//! | --- | --- | --- |
//! | `async` | yes | `CST328::new(i2c, delay)` over `embedded_hal_async::i2c::I2c` + `embedded_hal_async::delay::DelayNs`, plus an optional reset pin via `.with_reset()`. |
//! | `blocking` | no | `CST328::new(i2c, delay)` over `embedded_hal::i2c::I2c` + `embedded_hal::delay::DelayNs`, plus an optional reset pin. |
//! | `defmt` | no | Derives `defmt::Format` on the public types for logging. |
//!
//! `async` and `blocking` both export a `CST328` type at the crate root and
//! are mutually exclusive — enabling both is a compile error. Use
//! `default-features = false, features = ["blocking"]` to switch to the sync
//! driver. Bring your own async delay impl for `async` (e.g. `embassy_time::Delay`
//! if you already depend on embassy-time) — this crate doesn't hardcode one.
//!
//! ```rust,ignore
//! use cst328::{CST328, RunMode};
//!
//! let mut driver = CST328::new(i2c, delay); // add `.with_reset(rst_pin)` if wired up
//! driver.init().await?;
//!
//! for point in driver.touches().await?.iter().flatten() {
//!     // point.track_id, point.x, point.y, point.area (pressure)
//! }
//! ```
//!
//! See the [repository README](https://github.com/ScripTerasu/cst328-touch-driver)
//! for complete async and blocking examples, wiring notes, and the list of
//! `RunMode` variants that are unverified against real hardware.
//!
//! [waveshare]: https://github.com/waveshareteam/Waveshare-ESP32-components/blob/main/display/touch/esp_lcd_touch_cst328/esp_lcd_touch_cst328.c
//! [esphome]: https://github.com/esphome/esphome/blob/dev/esphome/components/cst328/touchscreen/cst328_touchscreen.cpp

#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!(
    "features `async` and `blocking` both export a `CST328` type at the crate root and are \
    mutually exclusive; enable exactly one (`default-features = false, features = [\"blocking\"]` \
    for the sync driver, or just `features = [\"async\"]`, which is already the default)."
);

pub mod error;
pub mod info;
pub mod mode;
pub mod registers;
pub mod reset_pin;
pub mod types;

pub use error::Error;
pub use info::{ChipInfo, Point};
pub use mode::RunMode;
pub use reset_pin::NoResetPin;
pub use types::{DisplayMapping, Orientation, TouchConfig};

#[cfg(any(feature = "async", feature = "blocking"))]
mod protocol;

#[cfg(any(feature = "async", feature = "blocking"))]
mod driver;

#[cfg(any(feature = "async", feature = "blocking"))]
pub use driver::CST328;
