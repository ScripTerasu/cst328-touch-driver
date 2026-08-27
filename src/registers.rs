//! Register map for the "classic" CST328/CST3530 protocol: 16-bit big-endian
//! register addresses, as used by Waveshare's official
//! `esp_lcd_touch_cst328` component and ESPHome's `cst328` component — both
//! written for the exact chip on the Waveshare ESP32-S3-Touch-LCD-2.8 board.
//!
//! This is deliberately *not* ported from SensorLib: SensorLib has no
//! `TouchDrvCST328` at all — its `CST328_SLAVE_ADDRESS` alias resolves to
//! `TouchDrvCST3530`, which speaks an unrelated 4-byte command-wrapper
//! protocol (different register numbering, different touch-packet nibble
//! order, different ack mechanism). See the README for the full rationale.

/// The controller's fixed 7-bit I²C address.
pub const CST328_SLAVE_ADDRESS: u8 = 0x1A;

/// Touch report register: 27 bytes starting here hold the finger-count byte
/// (at offset 5) and up to 5 point records. Also the write target for the
/// sync/ack byte ([`CST328_SYNC_BYTE`]) after a report is consumed.
pub const REG_READ: u16 = 0xD000;
/// Single byte: low nibble is the number of active touch points (0-5). This
/// is `REG_READ + 5`, i.e. offset 5 of the touch report block, not an
/// independent report — but it's also the write target for the ack/clear
/// byte after a report is consumed.
pub const REG_FINGER_NUM: u16 = 0xD005;

/// Work-mode register: writing (zero-length payload) enters debug/info mode,
/// which exposes the `0xD1Fx`/`0xD2xx` attribute registers below. Used by
/// `get_attribute()` before reading chip attributes, mirroring
/// `esp_lcd_touch_cst328`/ESPHome's `continue_setup_()`.
pub const REG_DEBUG_INFO_MODE: u16 = 0xD101;
/// Reset-mode selector. Declared by both reference drivers but never written
/// by either — unverified against real hardware.
pub const REG_RESET_MODE: u16 = 0xD102;
/// Debug recalibration mode. Declared but never written by either reference
/// driver — unverified against real hardware.
pub const REG_DEBUG_RECALIBRATION_MODE: u16 = 0xD104;
/// Deep-sleep mode. Declared but never written by either reference driver —
/// unverified against real hardware. Both reference drivers rely on the RST
/// pin (held asserted) for power-down instead of a software sleep command.
pub const REG_DEEP_SLEEP_MODE: u16 = 0xD105;
/// Debug point mode. Declared but never written by either reference driver —
/// unverified against real hardware.
pub const REG_DEBUG_POINT_MODE: u16 = 0xD108;
/// Work-mode register: writing (zero-length payload) returns to normal
/// touch-reporting mode. Written at the end of `get_attribute()`, mirroring
/// `continue_setup_()`'s return-to-normal-mode step.
pub const REG_NORMAL_MODE: u16 = 0xD109;
/// Raw-data debug mode. Declared but never written by either reference
/// driver — unverified against real hardware.
pub const REG_DEBUG_RAWDATA_MODE: u16 = 0xD10A;
/// Diff debug mode. Declared but never written by either reference driver —
/// unverified against real hardware.
pub const REG_DEBUG_DIFF_MODE: u16 = 0xD10D;
/// Factory test mode. Declared but never written by either reference
/// driver — unverified against real hardware.
pub const REG_DEBUG_FACTORY_MODE: u16 = 0xD119;
/// A second factory test mode. Declared but never written by either
/// reference driver — unverified against real hardware.
pub const REG_DEBUG_FACTORY_MODE_2: u16 = 0xD120;

/// TX channel count, part of the debug-info block. Logged only by the
/// reference drivers, not parsed into a typed field here.
pub const REG_DEBUG_INFO_TP_NTX: u16 = 0xD1F4;
/// RX channel count, part of the debug-info block. Logged only by the
/// reference drivers, not parsed into a typed field here.
pub const REG_DEBUG_INFO_TP_NRX: u16 = 0xD1F6;
/// Key/button number, part of the debug-info block. Logged only by the
/// reference drivers, not parsed into a typed field here.
pub const REG_DEBUG_INFO_KEY_NUM: u16 = 0xD1F7;
/// Panel resolution: 4-byte read yields X (bytes 0-1, little-endian u16)
/// then Y (bytes 2-3, little-endian u16).
pub const REG_RESOLUTION: u16 = 0xD1F8;
/// Boot time + firmware CRC: 4-byte read yields boot time (bytes 0-1,
/// undocumented semantics, logged only) then firmware CRC (bytes 2-3,
/// little-endian u16), which `get_attribute()` requires to equal `0xCACA`.
pub const REG_CHECK_CODE: u16 = 0xD1FC;
/// Project ID + chip ID: 4-byte read yields project ID (bytes 0-1,
/// little-endian u16) then chip ID (bytes 2-3, little-endian u16). No known
/// chip-ID value distinguishes CST328 from CST3530 — see `ChipInfo::chip_id`.
pub const REG_CHIP_TYPE: u16 = 0xD204;
/// Firmware version: 4-byte read yields build number (bytes 0-1,
/// little-endian u16), minor version (byte 2), then major version (byte 3).
pub const REG_FW_VERSION: u16 = 0xD208;

/// Sync/ack byte written to [`REG_READ`] to arm the touch-report mechanism
/// (once during `get_attribute()`) and re-arm it after every report is
/// consumed (alongside clearing [`REG_FINGER_NUM`]).
pub const CST328_SYNC_BYTE: u8 = 0xAB;

/// Maximum simultaneous touch contacts the controller reports (and this
/// driver decodes).
pub const MAX_FINGER_NUM: usize = 5;
/// Bytes in a `REG_READ` touch report: 5 points x 5 bytes each, plus 2 extra
/// bytes folded into the first point's wider 7-byte stride (see
/// `protocol::decode_touch_report`).
pub const TOUCH_DATA_SIZE: usize = MAX_FINGER_NUM * 5 + 2;
