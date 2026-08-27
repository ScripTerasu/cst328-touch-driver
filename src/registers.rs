//! Register map for the "classic" CST328/CST3530 protocol: 16-bit big-endian
//! register addresses, as used by Waveshare's official
//! `esp_lcd_touch_cst328` component and ESPHome's `cst328` component — both
//! written for the exact chip on the Waveshare ESP32-S3-Touch-LCD-2.8 board.
//! Cross-checked against Hynitron's official CST328 datasheet (section 12,
//! "寄存器附录"/register appendix — see `docs/CST328_Datasheet_zh_V2.2.pdf`),
//! which confirms this map byte-for-byte and adds a few registers neither
//! reference driver happens to use (noted per-constant below).
//!
//! This is deliberately *not* ported from SensorLib: SensorLib has no
//! `TouchDrvCST328` at all — its `CST328_SLAVE_ADDRESS` alias resolves to
//! `TouchDrvCST3530`, which speaks an unrelated 4-byte command-wrapper
//! protocol (different register numbering, different touch-packet nibble
//! order, different ack mechanism). See the README for the full rationale.
//!
//! Hynitron's CST3530 datasheet (`docs/CST3530_Datasheet_V1.0.pdf`) confirms
//! the same default I2C address (8-bit `0x34`/`0x35`, i.e. 7-bit `0x1A`) but
//! has no register appendix at all — CST3530 support in this crate remains
//! an assumption that it speaks the same register protocol as CST328
//! (inherited from SensorLib's address-alias precedent), not something
//! independently confirmed. The CST3530 datasheet also documents a larger,
//! meaningfully different chip (30 channels vs. CST328's 28, up to 10 real
//! touch points vs. 5, a wider `VDDA` range) — if [`MAX_FINGER_NUM`]/
//! [`TOUCH_DATA_SIZE`] turn out to not fit genuine CST3530 silicon, that's
//! the most likely reason.

/// The controller's fixed 7-bit I²C address.
///
/// Confirmed in both the CST328 and CST3530 datasheets: the default 8-bit
/// address (including the R/W bit) is `0x34`/`0x35`, i.e. this 7-bit value.
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
/// `System_Reset` flag: soft-resets the whole chip. Documented by the
/// official CST328 datasheet (section 12) but not written by either
/// reference driver (both use the hardware RST pin instead) — datasheet-
/// confirmed to work, just not exercised by the sources this crate ports.
pub const REG_RESET_MODE: u16 = 0xD102;
/// `Redo_Calibration` flag: reinitializes the touch algorithm. Documented by
/// the official CST328 datasheet (section 12) but not written by either
/// reference driver — datasheet-confirmed to work, just not exercised by the
/// sources this crate ports.
pub const REG_DEBUG_RECALIBRATION_MODE: u16 = 0xD104;
/// Deep-sleep mode. Documented by the official CST328 datasheet (section 12)
/// as a real, working "enter sleep" command — but not written by either
/// reference driver, which both rely on the RST pin (held asserted) for
/// power-down instead. Datasheet-confirmed to work, just not exercised by
/// the sources this crate ports.
pub const REG_DEEP_SLEEP_MODE: u16 = 0xD105;
/// `ENUM_MODE_DEBUG_POINTS`: enters debug point-report mode. Documented by
/// the official CST328 datasheet (section 12) but not written by either
/// reference driver — datasheet-confirmed to work, just not exercised by the
/// sources this crate ports.
pub const REG_DEBUG_POINT_MODE: u16 = 0xD108;
/// Work-mode register: writing (zero-length payload) returns to normal
/// touch-reporting mode. Written at the end of `get_attribute()`, mirroring
/// `continue_setup_()`'s return-to-normal-mode step.
pub const REG_NORMAL_MODE: u16 = 0xD109;
/// `ENUM_MODE_DEBUG_RAWDATA`: enters raw-data debug mode. Documented by the
/// official CST328 datasheet (section 12) but not written by either
/// reference driver — datasheet-confirmed to work, just not exercised by the
/// sources this crate ports.
pub const REG_DEBUG_RAWDATA_MODE: u16 = 0xD10A;
/// `ENUM_MODE_DEBUG_WRITE`: enters debug write mode. Documented by the
/// official CST328 datasheet (section 12); not present in either reference
/// driver's source at all (this crate's register set matched the two
/// reference drivers before cross-checking against the datasheet, which is
/// where this one came from) — unverified against real hardware.
pub const REG_DEBUG_WRITE_MODE: u16 = 0xD10B;
/// `ENUM_MODE_DEBUG_CALIBRATION`: enters a redo-calibration debug mode.
/// Documented by the official CST328 datasheet (section 12) as distinct
/// from the [`REG_DEBUG_RECALIBRATION_MODE`] flag; not present in either
/// reference driver's source — unverified against real hardware.
pub const REG_DEBUG_CALIBRATION_MODE: u16 = 0xD10C;
/// `ENUM_MODE_DEBUG_DIFF`: enters diff debug mode. Documented by the
/// official CST328 datasheet (section 12) but not written by either
/// reference driver — datasheet-confirmed to work, just not exercised by the
/// sources this crate ports.
pub const REG_DEBUG_DIFF_MODE: u16 = 0xD10D;
/// `ENUM_MODE_FACTORY`: enters factory test mode. Documented by the official
/// CST328 datasheet (section 12) but not written by either reference
/// driver — datasheet-confirmed to work, just not exercised by the sources
/// this crate ports.
pub const REG_DEBUG_FACTORY_MODE: u16 = 0xD119;
/// A second factory test mode, declared by Waveshare's official driver.
/// **Not present in the official CST328 datasheet's register appendix** —
/// unlike every other register in this file, this one has no datasheet
/// confirmation at all. Treat it as the least-trustworthy constant here.
pub const REG_DEBUG_FACTORY_MODE_2: u16 = 0xD120;

/// Panel resolution: 4-byte read yields X (bytes 0-1, little-endian u16)
/// then Y (bytes 2-3, little-endian u16).
pub const REG_RESOLUTION: u16 = 0xD1F8;
/// Boot timer + firmware CRC: 4-byte read yields the boot timer (bytes 0-1,
/// little-endian u16 — labeled `BOOT_TIMER` in the official datasheet, whose
/// units/semantics aren't documented beyond the name, so logged only) then a
/// fixed `0xCACA` marker (bytes 2-3), which `get_attribute()` requires.
pub const REG_CHECK_CODE: u16 = 0xD1FC;
/// Project ID + chip ID: 4-byte read yields project ID (bytes 0-1,
/// little-endian u16) then chip ID (bytes 2-3, little-endian u16). No known
/// chip-ID value distinguishes CST328 from CST3530 — see `ChipInfo::chip_id`.
pub const REG_CHIP_TYPE: u16 = 0xD204;
/// Firmware version: 4-byte read yields build number (bytes 0-1,
/// little-endian u16), minor version (byte 2), then major version (byte 3).
pub const REG_FW_VERSION: u16 = 0xD208;
/// Firmware checksum: 4-byte read yields checksum bytes 0-1 (`checksum_L`)
/// then bytes 2-3 (`checksum_H`), both little-endian halves of one 32-bit
/// value. Documented by the official CST328 datasheet (section 12) but not
/// read by either reference driver this crate otherwise ports — informational
/// only, not validated by `get_attribute()`.
pub const REG_FW_CHECKSUM: u16 = 0xD20C;

/// Sync/ack byte written to [`REG_READ`] to arm the touch-report mechanism
/// (once during `get_attribute()`) and re-arm it after every report is
/// consumed (alongside clearing [`REG_FINGER_NUM`]).
///
/// The same byte value also shows up *read-only*, chip-populated, at offset
/// 6 of every touch report (`REG_READ + 6`) — the official CST328 datasheet
/// documents that offset as a fixed `0xAB` marker. `decode_touch_report()`
/// checks it against this same constant as a frame-validity check.
pub const CST328_SYNC_BYTE: u8 = 0xAB;

/// Maximum simultaneous touch contacts the controller reports (and this
/// driver decodes).
pub const MAX_FINGER_NUM: usize = 5;
/// Bytes in a `REG_READ` touch report: 5 points x 5 bytes each, plus 2 extra
/// bytes folded into the first point's wider 7-byte stride (see
/// `protocol::decode_touch_report`).
pub const TOUCH_DATA_SIZE: usize = MAX_FINGER_NUM * 5 + 2;
