/// Supported run modes for the CST328/CST3530 controller.
///
/// Only [`RunMode::Normal`] and [`RunMode::DebugInfo`] are exercised by
/// Waveshare's official `esp_lcd_touch_cst328` component or ESPHome's
/// `cst328` component — the reference drivers this crate is ported from.
/// Every other variant is declared as a register by both reference drivers
/// but never written by either; they're mapped here by register-naming
/// convention only. Treat those as unproven until validated against real
/// hardware.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Normal touch-reporting mode. Written at the end of `get_attribute()`
    /// to leave debug/info mode; also available directly via `set_mode()`.
    Normal,
    /// Debug/info mode, exposing the `0xD1Fx`/`0xD2xx` attribute registers.
    /// Written at the start of `get_attribute()`.
    DebugInfo,
    /// Unverified: declared but never written by either reference driver.
    Reset,
    /// Unverified: declared but never written by either reference driver.
    DebugRecalibration,
    /// Unverified: declared but never written by either reference driver.
    /// Both reference drivers power down via the RST pin instead.
    DeepSleep,
    /// Unverified: declared but never written by either reference driver.
    DebugPoint,
    /// Unverified: declared but never written by either reference driver.
    DebugRawData,
    /// Unverified: declared but never written by either reference driver.
    DebugDiff,
    /// Unverified: declared but never written by either reference driver.
    Factory,
    /// Unverified: declared but never written by either reference driver.
    Factory2,
}
