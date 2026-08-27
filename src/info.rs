/// A detected touch point, already transformed according to `TouchConfig`.
///
/// Unlike the CST92xx family, CST328/CST3530 reports a real per-point
/// pressure/weight byte — `area` here is that raw value, not always `0`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Point {
    pub track_id: u8,
    pub x: u16,
    pub y: u16,
    pub area: u16,
}

/// Chip metadata, discovered and validated by `get_attribute()`.
/// Read-only for callers — this is hardware state, not configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ChipInfo {
    /// Raw chip ID read from `REG_CHIP_TYPE` (`0xD204`, bytes 2-3).
    ///
    /// No known chip-ID value distinguishes CST328 from CST3530 in either
    /// reference driver this crate is ported from (Waveshare's official
    /// `esp_lcd_touch_cst328` or ESPHome's `cst328` component) — both read
    /// this field only to log it, never to gate on a specific value. Treat
    /// it as informational; `model_name()` can't use it to disambiguate.
    pub chip_id: u16,
    /// Raw project ID read from `REG_CHIP_TYPE` (`0xD204`, bytes 0-1).
    pub project_id: u16,
    pub resolution_x: u16,
    pub resolution_y: u16,
    /// Firmware CRC read from `REG_CHECK_CODE` (`0xD1FC`, bytes 2-3).
    /// `get_attribute()` requires this to equal `0xCACA` to succeed.
    pub fw_crc: u16,
    /// Firmware major version, from `REG_FW_VERSION` (`0xD208`, byte 3).
    pub fw_major: u8,
    /// Firmware minor version, from `REG_FW_VERSION` (`0xD208`, byte 2).
    pub fw_minor: u8,
    /// Firmware build number, from `REG_FW_VERSION` (`0xD208`, bytes 0-1).
    pub fw_build: u16,
    /// Firmware checksum, from `REG_FW_CHECKSUM` (`0xD20C`). Documented by
    /// the official CST328 datasheet but not read (or validated against
    /// anything) by either reference driver this crate otherwise ports —
    /// informational only.
    pub fw_checksum: u32,
}

impl ChipInfo {
    /// Model name for this chip family.
    ///
    /// Always returns `"CST328/CST3530"` — see [`ChipInfo::chip_id`] for why
    /// this driver can't disambiguate the two from any register value known
    /// to either reference driver.
    pub fn model_name(&self) -> &'static str {
        "CST328/CST3530"
    }
}
