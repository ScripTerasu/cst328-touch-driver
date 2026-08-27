//! Shared `CST328` driver body for both the `async` and `blocking` features.
//!
//! `async` and `blocking` are mutually exclusive (enforced in `lib.rs`), so of the
//! two `maybe_async!`/`maybe_await!` definitions below, only the one matching the
//! active feature is ever compiled. Each rewrites the driver body written once
//! here — written in blocking style, as plain `fn`s with no `.await` — into the
//! async or sync version, so an orchestration fix only has to be made in one
//! place instead of drifting between two hand-mirrored files. This module
//! covers the I/O orchestration `protocol.rs` deliberately leaves out.

#[cfg(feature = "blocking")]
use embedded_hal::delay::DelayNs;
#[cfg(feature = "blocking")]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::delay::DelayNs;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

use embedded_hal::digital::OutputPin;

use crate::error::Error;
use crate::info::{ChipInfo, Point};
use crate::mode::RunMode;
use crate::protocol;
use crate::registers::{
    CST328_SLAVE_ADDRESS, CST328_SYNC_BYTE, MAX_FINGER_NUM, REG_CHECK_CODE, REG_CHIP_TYPE,
    REG_DEBUG_INFO_MODE, REG_FINGER_NUM, REG_FW_VERSION, REG_NORMAL_MODE, REG_READ, REG_RESOLUTION,
    TOUCH_DATA_SIZE,
};
use crate::reset_pin::NoResetPin;
use crate::types::TouchConfig;

/// Turns a `fn`/`pub fn` item written below into `async fn`/`pub async fn`.
#[cfg(feature = "async")]
macro_rules! maybe_async {
    ($(#[$attr:meta])* pub fn $($rest:tt)*) => { $(#[$attr])* pub async fn $($rest)* };
    ($(#[$attr:meta])* fn $($rest:tt)*) => { $(#[$attr])* async fn $($rest)* };
}
/// Leaves a `fn`/`pub fn` item written below untouched.
#[cfg(feature = "blocking")]
macro_rules! maybe_async {
    ($(#[$attr:meta])* pub fn $($rest:tt)*) => { $(#[$attr])* pub fn $($rest)* };
    ($(#[$attr:meta])* fn $($rest:tt)*) => { $(#[$attr])* fn $($rest)* };
}

/// Appends `.await` to `$e`.
#[cfg(feature = "async")]
macro_rules! maybe_await {
    ($e:expr) => {
        $e.await
    };
}
/// Evaluates `$e` as-is, with no `.await`.
#[cfg(feature = "blocking")]
macro_rules! maybe_await {
    ($e:expr) => {
        $e
    };
}

/// CST328/CST3530 controller driver, generic over blocking `embedded-hal` or
/// `embedded-hal-async` I²C depending on which of the `async`/`blocking`
/// features is enabled (see the crate-level docs).
pub struct CST328<I2C, DELAY, RST = NoResetPin> {
    i2c: I2C,
    delay: DELAY,
    rst: RST,
    config: TouchConfig,
    chip_info: ChipInfo,
}

impl<I2C, E, DELAY> CST328<I2C, DELAY, NoResetPin>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
{
    /// Create a new driver without a dedicated reset pin.
    ///
    /// `i2c` must provide exclusive ownership of the bus and implement the 7-bit
    /// slave address the CST328/CST3530 controller listens on. `delay` is used
    /// for reset timing. Use `.with_reset()` to attach a real `RST` line if one
    /// is wired up.
    pub fn new(i2c: I2C, delay: DELAY) -> Self {
        Self {
            i2c,
            delay,
            rst: NoResetPin,
            config: TouchConfig::default(),
            chip_info: ChipInfo::default(),
        }
    }
}

impl<I2C, E, DELAY, RST> CST328<I2C, DELAY, RST>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
    RST: OutputPin,
{
    /// Attach a hardware reset pin, replacing the no-op default.
    pub fn with_reset<RST2: OutputPin>(self, rst: RST2) -> CST328<I2C, DELAY, RST2> {
        CST328 {
            i2c: self.i2c,
            delay: self.delay,
            rst,
            config: self.config,
            chip_info: self.chip_info,
        }
    }

    /// Override the touch coordinate transform (orientation/display mapping).
    pub fn with_config(mut self, config: TouchConfig) -> Self {
        self.config = config;
        self
    }

    /// Take ownership of the I2C bus, delay provider, and reset pin.
    ///
    /// Useful when you want to re-use the bus for other devices after the driver is dropped.
    pub fn into_inner(self) -> (I2C, DELAY, RST) {
        (self.i2c, self.delay, self.rst)
    }

    maybe_async! {
        /// Initialize the controller (reset + attribute read) and validate its firmware CRC.
        pub fn init(&mut self) -> Result<(), Error<E>> {
            maybe_await!(self.get_attribute())?;

            #[cfg(feature = "defmt")]
            defmt::debug!("Touch type:{}", self.model_name());
            Ok(())
        }
    }

    maybe_async! {
        /// Pulse the reset pin (if any) and wait for the controller to come back up.
        ///
        /// With the default `NoResetPin` this is just the settle delays; with a real `RST`
        /// pin attached via `.with_reset()`, the pin is pulsed low first. Timing matches
        /// ESPHome's `cst328` component (the clearer of the two reference drivers on this
        /// point): ensure the pin starts released (high), wait 50 ms, assert low for 5 ms,
        /// release high, then wait 300 ms for the controller to come back up before any I2C.
        pub fn reset(&mut self) {
            let _ = self.rst.set_high();
            maybe_await!(self.delay.delay_ms(50));
            let _ = self.rst.set_low();
            maybe_await!(self.delay.delay_ms(5));
            let _ = self.rst.set_high();
            maybe_await!(self.delay.delay_ms(300));
        }
    }

    maybe_async! {
        /// Read controller metadata (firmware CRC, resolution, chip/project ID, firmware
        /// version) and validate the firmware CRC.
        ///
        /// Mirrors ESPHome's `cst328` component `continue_setup_()`: enter debug/info mode,
        /// read the attribute registers, return to normal mode, then discard a stale byte
        /// from the touch-report register and arm the sync/ack byte so `touches()` sees a
        /// clean first report.
        pub fn get_attribute(&mut self) -> Result<(), Error<E>> {
            maybe_await!(self.reset());

            maybe_await!(self.write(&REG_DEBUG_INFO_MODE.to_be_bytes()))?;

            let mut buffer = [0u8; 4];
            maybe_await!(self.write_read(&REG_CHECK_CODE.to_be_bytes(), &mut buffer))?;
            let fw_crc = u16::from_le_bytes([buffer[2], buffer[3]]);

            #[cfg(feature = "defmt")]
            defmt::info!("Firmware CRC: {=u16:#06X}", fw_crc);

            if !protocol::validate_fw_crc(fw_crc) {
                #[cfg(feature = "defmt")]
                defmt::error!("Firmware CRC mismatch, expected 0xCACA");
                return Err(Error::InvalidCheckCode);
            }
            self.chip_info.fw_crc = fw_crc;

            maybe_await!(self.write_read(&REG_CHIP_TYPE.to_be_bytes(), &mut buffer))?;
            self.chip_info.project_id = u16::from_le_bytes([buffer[0], buffer[1]]);
            self.chip_info.chip_id = u16::from_le_bytes([buffer[2], buffer[3]]);

            #[cfg(feature = "defmt")]
            defmt::info!(
                "Chip ID={=u16:#06X}, Project ID={=u16:#06X}",
                self.chip_info.chip_id,
                self.chip_info.project_id
            );

            maybe_await!(self.write_read(&REG_FW_VERSION.to_be_bytes(), &mut buffer))?;
            self.chip_info.fw_build = u16::from_le_bytes([buffer[0], buffer[1]]);
            self.chip_info.fw_minor = buffer[2];
            self.chip_info.fw_major = buffer[3];

            #[cfg(feature = "defmt")]
            defmt::info!(
                "Firmware v{=u8}.{=u8} build {=u16}",
                self.chip_info.fw_major,
                self.chip_info.fw_minor,
                self.chip_info.fw_build
            );

            maybe_await!(self.write_read(&REG_RESOLUTION.to_be_bytes(), &mut buffer))?;
            self.chip_info.resolution_x = u16::from_le_bytes([buffer[0], buffer[1]]);
            self.chip_info.resolution_y = u16::from_le_bytes([buffer[2], buffer[3]]);

            #[cfg(feature = "defmt")]
            defmt::info!(
                "Chip resolution X={=u16} Y={=u16}",
                self.chip_info.resolution_x,
                self.chip_info.resolution_y
            );

            maybe_await!(self.write(&REG_NORMAL_MODE.to_be_bytes()))?;

            let reg_read_bytes = REG_READ.to_be_bytes();
            let mut discard = [0u8; 1];
            maybe_await!(self.write_read(&reg_read_bytes, &mut discard))?;
            maybe_await!(self.arm_sync_byte())?;

            Ok(())
        }
    }

    /// Chip metadata discovered by the last successful `get_attribute()`/`init()` call.
    pub fn chip_info(&self) -> ChipInfo {
        self.chip_info
    }

    /// Return the model string for this chip family.
    ///
    /// Always `"CST328/CST3530"` — see [`ChipInfo::chip_id`] for why this
    /// driver can't tell the two apart from any register value.
    pub fn model_name(&self) -> &'static str {
        self.chip_info.model_name()
    }

    maybe_async! {
        /// Switch to a controller run mode (normal, debug/info, factory test, etc.).
        ///
        /// Writes a zero-length payload to the mode's work-mode register — unlike CST92xx,
        /// no confirmation/status-echo register is known for this protocol, so there's no
        /// handshake or retry loop here. See [`RunMode`] for which variants are actually
        /// exercised by the reference drivers versus mapped by naming convention only.
        pub fn set_mode(&mut self, mode: RunMode) -> Result<(), Error<E>> {
            let reg = protocol::mode_register(mode);
            maybe_await!(self.write(&reg.to_be_bytes()))?;

            #[cfg(feature = "defmt")]
            defmt::debug!("set_mode -> {:?}", mode);

            Ok(())
        }
    }

    maybe_async! {
        /// Write raw bytes (register + payload) to the controller.
        ///
        /// Uses the fixed slave address `CST328_SLAVE_ADDRESS` so callers can always pass
        /// register+payload bytes directly.
        fn write(&mut self, write: &[u8]) -> Result<(), Error<E>> {
            maybe_await!(self.i2c.write(CST328_SLAVE_ADDRESS, write)).map_err(Error::I2C)
        }
    }

    maybe_async! {
        /// Write bytes and then read a response without leaving command mode.
        ///
        /// Performs a single `write_read` transaction that keeps the bus busy until the controller responds.
        fn write_read(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), Error<E>> {
            maybe_await!(self.i2c.write_read(CST328_SLAVE_ADDRESS, write, read))
                .map_err(Error::I2C)
        }
    }

    maybe_async! {
        /// Write the sync/ack byte to `REG_READ`, arming the controller to report the next
        /// touch frame. Called once during `get_attribute()` and after every `touches()` read.
        fn arm_sync_byte(&mut self) -> Result<(), Error<E>> {
            let reg_bytes = REG_READ.to_be_bytes();
            let mut write_buffer = [0u8; 3];
            write_buffer[0] = reg_bytes[0];
            write_buffer[1] = reg_bytes[1];
            write_buffer[2] = CST328_SYNC_BYTE;
            maybe_await!(self.write(&write_buffer))
        }
    }

    maybe_async! {
        /// Read the latest touch report from `REG_READ` and translate it into `Point`s.
        ///
        /// Acknowledges every read regardless of content — clearing `REG_FINGER_NUM` and
        /// re-arming the sync byte at `REG_READ` — matching both ESPHome's `cst328` component
        /// and SensorLib's `TouchDrvCST3530::getTouchPoints()` (which sends its `CLEAR_COMMAND`
        /// unconditionally). This intentionally does not replicate the CST92xx driver's
        /// all-zero-buffer skip, which neither reference implementation here does either.
        pub fn touches(&mut self) -> Result<[Option<Point>; MAX_FINGER_NUM], Error<E>> {
            let mut buffer = [0u8; TOUCH_DATA_SIZE];
            let reg_bytes = REG_READ.to_be_bytes();

            maybe_await!(self.write_read(&reg_bytes, &mut buffer))?;

            let panel_resolution = (self.chip_info.resolution_x, self.chip_info.resolution_y);
            let points = protocol::decode_touch_report(&buffer, &self.config, panel_resolution);

            let mut clear_buffer = [0u8; 3];
            let finger_num_bytes = REG_FINGER_NUM.to_be_bytes();
            clear_buffer[0] = finger_num_bytes[0];
            clear_buffer[1] = finger_num_bytes[1];
            clear_buffer[2] = 0x00;
            maybe_await!(self.write(&clear_buffer))?;

            maybe_await!(self.arm_sync_byte())?;

            Ok(points)
        }
    }
}
