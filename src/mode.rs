/// Supported run modes for the CST328/CST3530 controller.
///
/// Only [`RunMode::Normal`] and [`RunMode::DebugInfo`] are exercised by
/// Waveshare's official `esp_lcd_touch_cst328` component or ESPHome's
/// `cst328` component — the reference drivers this crate is ported from.
/// Every other variant falls into one of two confidence tiers, noted per
/// variant below:
///
/// - **Datasheet-confirmed**: documented with a real description in
///   Hynitron's official CST328 datasheet (section 12, register appendix),
///   just not exercised by either reference driver's source.
/// - **Unverified**: not in the official datasheet's register appendix at
///   all (only [`RunMode::Factory2`]), or in the appendix but with no
///   independent confirmation this driver's Rust port of the write sequence
///   is correct on real hardware.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Normal touch-reporting mode. Written at the end of `get_attribute()`
    /// to leave debug/info mode; also available directly via `set_mode()`.
    Normal,
    /// Debug/info mode, exposing the `0xD1Fx`/`0xD2xx` attribute registers.
    /// Written at the start of `get_attribute()`.
    DebugInfo,
    /// `System_Reset`: soft-resets the whole chip. Datasheet-confirmed; not
    /// written by either reference driver, which both use the hardware RST
    /// pin instead.
    Reset,
    /// `Redo_Calibration`: reinitializes the touch algorithm.
    /// Datasheet-confirmed; not written by either reference driver.
    DebugRecalibration,
    /// Deep-sleep mode. Datasheet-confirmed as a real "enter sleep" command;
    /// not written by either reference driver, which both power down via the
    /// RST pin instead.
    DeepSleep,
    /// `ENUM_MODE_DEBUG_POINTS`. Datasheet-confirmed; not written by either
    /// reference driver.
    DebugPoint,
    /// `ENUM_MODE_DEBUG_RAWDATA`. Datasheet-confirmed; not written by either
    /// reference driver.
    DebugRawData,
    /// `ENUM_MODE_DEBUG_WRITE`. Datasheet-confirmed, but not present in
    /// either reference driver's source at all — unverified.
    DebugWrite,
    /// `ENUM_MODE_DEBUG_CALIBRATION`, distinct from [`RunMode::DebugRecalibration`].
    /// Datasheet-confirmed, but not present in either reference driver's
    /// source at all — unverified.
    DebugCalibration,
    /// `ENUM_MODE_DEBUG_DIFF`. Datasheet-confirmed; not written by either
    /// reference driver.
    DebugDiff,
    /// `ENUM_MODE_FACTORY`. Datasheet-confirmed; not written by either
    /// reference driver.
    Factory,
    /// A second factory test mode declared by Waveshare's official driver.
    /// **Unverified**: absent from the official datasheet's register
    /// appendix entirely, unlike every other variant here.
    Factory2,
}
