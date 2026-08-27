//! Pure, I/O-free pieces of the CST328/CST3530 protocol shared between the
//! blocking and async drivers.
//!
//! Everything here takes already-read bytes and returns a decoded/validated
//! result — no I2C, no delays. Keeping these in one place means a protocol
//! fix (or bug) only has to be made once instead of drifting between the two
//! driver implementations.

use crate::info::Point;
use crate::mode::RunMode;
use crate::registers::{
    CST328_SYNC_BYTE, MAX_FINGER_NUM, REG_DEBUG_CALIBRATION_MODE, REG_DEBUG_DIFF_MODE,
    REG_DEBUG_FACTORY_MODE, REG_DEBUG_FACTORY_MODE_2, REG_DEBUG_INFO_MODE, REG_DEBUG_POINT_MODE,
    REG_DEBUG_RAWDATA_MODE, REG_DEBUG_RECALIBRATION_MODE, REG_DEBUG_WRITE_MODE,
    REG_DEEP_SLEEP_MODE, REG_NORMAL_MODE, REG_RESET_MODE,
};
use crate::types::TouchConfig;

/// Firmware CRC `get_attribute()` requires (at `REG_CHECK_CODE`, bytes 2-3).
const EXPECTED_FW_CRC: u16 = 0xCACA;

/// Validate the firmware CRC read from `REG_CHECK_CODE`, mirroring the
/// `fw_crc != CST328_FW_CRC` check in ESPHome's `cst328` component (the only
/// value-based validation either reference driver performs).
pub(crate) fn validate_fw_crc(fw_crc: u16) -> bool {
    fw_crc == EXPECTED_FW_CRC
}

/// The register `set_mode()` should write for a given [`RunMode`].
///
/// Every mode transition is a zero-length write to a work-mode register — no
/// confirmation/status-echo register is known for this protocol, so there's
/// no handshake/retry state to represent here.
pub(crate) fn mode_register(mode: RunMode) -> u16 {
    match mode {
        RunMode::Normal => REG_NORMAL_MODE,
        RunMode::DebugInfo => REG_DEBUG_INFO_MODE,
        RunMode::Reset => REG_RESET_MODE,
        RunMode::DebugRecalibration => REG_DEBUG_RECALIBRATION_MODE,
        RunMode::DeepSleep => REG_DEEP_SLEEP_MODE,
        RunMode::DebugPoint => REG_DEBUG_POINT_MODE,
        RunMode::DebugRawData => REG_DEBUG_RAWDATA_MODE,
        RunMode::DebugWrite => REG_DEBUG_WRITE_MODE,
        RunMode::DebugCalibration => REG_DEBUG_CALIBRATION_MODE,
        RunMode::DebugDiff => REG_DEBUG_DIFF_MODE,
        RunMode::Factory => REG_DEBUG_FACTORY_MODE,
        RunMode::Factory2 => REG_DEBUG_FACTORY_MODE_2,
    }
}

/// Decode a `REG_READ` report into touch points, applying `config`'s
/// coordinate transform.
///
/// `buffer` must be the full `TOUCH_DATA_SIZE`-byte (27) report. Byte 5's
/// low nibble is the active-point count; point 0 occupies bytes `0..5` with
/// the count byte and one padding byte folded into its 7-byte stride, and
/// points 1-4 each occupy a plain 5-byte stride after that — this asymmetric
/// layout (not a uniform 5-byte array) is what both Waveshare's official
/// `esp_lcd_touch_cst328` driver and ESPHome's `cst328` component decode, so
/// it's preserved here rather than "cleaned up". Byte 6 is validated against
/// [`CST328_SYNC_BYTE`] — the official CST328 datasheet documents that offset
/// as a fixed `0xAB` marker the chip populates in every report, which neither
/// reference driver checks but which gives a cheap way to reject a garbled
/// or stale read.
pub(crate) fn decode_touch_report(
    buffer: &[u8],
    config: &TouchConfig,
    panel_resolution: (u16, u16),
) -> [Option<Point>; MAX_FINGER_NUM] {
    let mut points: [Option<Point>; MAX_FINGER_NUM] = [None; MAX_FINGER_NUM];

    if buffer[6] != CST328_SYNC_BYTE {
        return points;
    }

    let touch_count = (buffer[5] & 0x0F) as usize;
    if touch_count == 0 || touch_count > MAX_FINGER_NUM {
        return points;
    }

    let mut data_idx = 0usize;
    for slot in points.iter_mut().take(touch_count) {
        if data_idx + 5 > buffer.len() {
            break;
        }
        let record = &buffer[data_idx..data_idx + 5];
        let id = record[0] >> 4;
        let raw_x = ((record[1] as u16) << 4) | ((record[3] >> 4) as u16);
        let raw_y = ((record[2] as u16) << 4) | ((record[3] & 0x0F) as u16);
        let pressure = record[4] as u16;
        let (x, y) = config.transform(panel_resolution, raw_x, raw_y);

        *slot = Some(Point {
            track_id: id,
            x,
            y,
            area: pressure,
        });

        // Point 0's record is followed by 2 extra bytes (the finger-count
        // byte at offset 5 and one padding byte) before point 1 starts;
        // every later point is a plain 5-byte stride.
        data_idx += if data_idx == 0 { 7 } else { 5 };
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_expected_crc() {
        assert!(validate_fw_crc(0xCACA));
        assert!(!validate_fw_crc(0x1234));
    }

    #[test]
    fn zero_touch_count_returns_no_points() {
        let mut buffer = [0u8; crate::registers::TOUCH_DATA_SIZE];
        buffer[6] = CST328_SYNC_BYTE;
        let points = decode_touch_report(&buffer, &TouchConfig::default(), (240, 320));
        assert!(points.iter().all(|p| p.is_none()));
    }

    #[test]
    fn touch_count_above_max_returns_no_points() {
        let mut buffer = [0u8; crate::registers::TOUCH_DATA_SIZE];
        buffer[5] = 0x06; // 6 > MAX_FINGER_NUM (5)
        buffer[6] = CST328_SYNC_BYTE;
        let points = decode_touch_report(&buffer, &TouchConfig::default(), (240, 320));
        assert!(points.iter().all(|p| p.is_none()));
    }

    #[test]
    fn mismatched_sync_byte_returns_no_points() {
        let mut buffer = [0u8; crate::registers::TOUCH_DATA_SIZE];
        buffer[5] = 0x01; // touch_count = 1, would otherwise decode a point
        buffer[6] = 0x00; // not CST328_SYNC_BYTE
        let points = decode_touch_report(&buffer, &TouchConfig::default(), (240, 320));
        assert!(points.iter().all(|p| p.is_none()));
    }

    #[test]
    fn decodes_single_point() {
        let mut buffer = [0u8; crate::registers::TOUCH_DATA_SIZE];
        // Point 0 record: id=1, raw_x=0x0A5=165, raw_y=0x147=327, pressure=0x32.
        buffer[0] = 0x10; // id=1 in high nibble
        buffer[1] = 0x0A; // x high byte
        buffer[2] = 0x14; // y high byte
        buffer[3] = 0x57; // x low nibble=5, y low nibble=7
        buffer[4] = 0x32; // pressure
        buffer[5] = 0x01; // touch_count = 1
        buffer[6] = CST328_SYNC_BYTE;

        let points = decode_touch_report(&buffer, &TouchConfig::default(), (240, 320));
        let point = points[0].unwrap();
        assert_eq!(point.track_id, 1);
        assert_eq!(point.x, (0x0Au16 << 4) | 0x05);
        assert_eq!(point.y, (0x14u16 << 4) | 0x07);
        assert_eq!(point.area, 0x32);
        assert!(points[1].is_none());
    }

    #[test]
    fn decodes_two_points_with_asymmetric_stride() {
        let mut buffer = [0u8; crate::registers::TOUCH_DATA_SIZE];
        // Point 0 at offset 0..5.
        buffer[0] = 0x00; // id=0
        buffer[1] = 0x01;
        buffer[2] = 0x02;
        buffer[3] = 0x00;
        buffer[4] = 0x00;
        buffer[5] = 0x02; // touch_count = 2
        buffer[6] = CST328_SYNC_BYTE;
        // Point 1 at offset 7..12 (0 + 7 stride from point 0).
        buffer[7] = 0x10; // id=1
        buffer[8] = 0x03;
        buffer[9] = 0x04;
        buffer[10] = 0x00;
        buffer[11] = 0x00;

        let points = decode_touch_report(&buffer, &TouchConfig::default(), (240, 320));
        assert_eq!(points[0].unwrap().track_id, 0);
        assert_eq!(points[0].unwrap().x, 0x01 << 4);
        assert_eq!(points[1].unwrap().track_id, 1);
        assert_eq!(points[1].unwrap().x, 0x03 << 4);
    }
}
