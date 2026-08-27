use embedded_hal_mock::eh1::delay::NoopDelay;
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use futures::executor::block_on;

use cst328::{CST328, Error, RunMode, registers};

const READ_LEN: usize = registers::TOUCH_DATA_SIZE;
const REG_READ_BYTES: [u8; 2] = registers::REG_READ.to_be_bytes();
const REG_FINGER_NUM_BYTES: [u8; 2] = registers::REG_FINGER_NUM.to_be_bytes();
const READ_REPORT_EMPTY: [u8; READ_LEN] = [0u8; READ_LEN];

const CLEAR_COMMAND: [u8; 3] = [REG_FINGER_NUM_BYTES[0], REG_FINGER_NUM_BYTES[1], 0x00];
const SYNC_COMMAND: [u8; 3] = [
    REG_READ_BYTES[0],
    REG_READ_BYTES[1],
    registers::CST328_SYNC_BYTE,
];

const REG_DEBUG_INFO_MODE_BYTES: [u8; 2] = registers::REG_DEBUG_INFO_MODE.to_be_bytes();
const REG_NORMAL_MODE_BYTES: [u8; 2] = registers::REG_NORMAL_MODE.to_be_bytes();
const REG_CHECK_CODE_BYTES: [u8; 2] = registers::REG_CHECK_CODE.to_be_bytes();
const REG_RESOLUTION_BYTES: [u8; 2] = registers::REG_RESOLUTION.to_be_bytes();
const REG_CHIP_TYPE_BYTES: [u8; 2] = registers::REG_CHIP_TYPE.to_be_bytes();
const REG_FW_VERSION_BYTES: [u8; 2] = registers::REG_FW_VERSION.to_be_bytes();

// fw_crc = 0xCACA (bytes[2..4], little-endian); bytes[0..2] is the undocumented boot-time field.
const CHECK_CODE_VALID: [u8; 4] = [0x11, 0x22, 0xCA, 0xCA];
// fw_crc = 0x1234, doesn't match the expected 0xCACA marker.
const CHECK_CODE_INVALID: [u8; 4] = [0x11, 0x22, 0x34, 0x12];
// resolution_x = 240, resolution_y = 320, both little-endian u16.
const RESOLUTION_VALID: [u8; 4] = [0xF0, 0x00, 0x40, 0x01];
// project_id = 0x1234, chip_id = 0x0328, both little-endian u16.
const CHIP_TYPE_VALID: [u8; 4] = [0x34, 0x12, 0x28, 0x03];
// fw_build = 0x0102 (little-endian), fw_minor = 0x05, fw_major = 0x03.
const FW_VERSION_VALID: [u8; 4] = [0x02, 0x01, 0x05, 0x03];
const DISCARD_BYTE: [u8; 1] = [0x00];

const ADDR: u8 = registers::CST328_SLAVE_ADDRESS;

fn attribute_expectations(check_code: &'static [u8; 4]) -> Vec<I2cTransaction> {
    vec![
        I2cTransaction::write(ADDR, REG_DEBUG_INFO_MODE_BYTES.to_vec()),
        I2cTransaction::write_read(ADDR, REG_CHECK_CODE_BYTES.to_vec(), check_code.to_vec()),
    ]
}

fn attribute_expectations_success() -> Vec<I2cTransaction> {
    let mut expectations = attribute_expectations(&CHECK_CODE_VALID);
    expectations.extend([
        I2cTransaction::write_read(ADDR, REG_CHIP_TYPE_BYTES.to_vec(), CHIP_TYPE_VALID.to_vec()),
        I2cTransaction::write_read(
            ADDR,
            REG_FW_VERSION_BYTES.to_vec(),
            FW_VERSION_VALID.to_vec(),
        ),
        I2cTransaction::write_read(
            ADDR,
            REG_RESOLUTION_BYTES.to_vec(),
            RESOLUTION_VALID.to_vec(),
        ),
        I2cTransaction::write(ADDR, REG_NORMAL_MODE_BYTES.to_vec()),
        I2cTransaction::write_read(ADDR, REG_READ_BYTES.to_vec(), DISCARD_BYTE.to_vec()),
        I2cTransaction::write(ADDR, SYNC_COMMAND.to_vec()),
    ]);
    expectations
}

#[test]
fn touches_empty_report_returns_no_points_async() {
    let expectations = [
        I2cTransaction::write_read(ADDR, REG_READ_BYTES.to_vec(), READ_REPORT_EMPTY.to_vec()),
        I2cTransaction::write(ADDR, CLEAR_COMMAND.to_vec()),
        I2cTransaction::write(ADDR, SYNC_COMMAND.to_vec()),
    ];
    let mut i2c = I2cMock::new(&expectations);

    let mut driver = CST328::new(i2c.clone(), NoopDelay::new());
    let touches = block_on(async { driver.touches().await.unwrap() });
    assert!(touches.iter().all(|point| point.is_none()));

    i2c.done();
}

#[test]
fn touches_parses_single_point_async() {
    let mut report = [0u8; READ_LEN];
    report[0] = 0x10; // track_id = 1
    report[1] = 0x0A;
    report[2] = 0x14;
    report[3] = 0x57;
    report[4] = 0x32; // pressure
    report[5] = 0x01; // touch_count = 1

    let expectations = [
        I2cTransaction::write_read(ADDR, REG_READ_BYTES.to_vec(), report.to_vec()),
        I2cTransaction::write(ADDR, CLEAR_COMMAND.to_vec()),
        I2cTransaction::write(ADDR, SYNC_COMMAND.to_vec()),
    ];
    let mut i2c = I2cMock::new(&expectations);

    let mut driver = CST328::new(i2c.clone(), NoopDelay::new());
    let touches = block_on(async { driver.touches().await.unwrap() });
    let point = touches[0].unwrap();
    assert_eq!(point.track_id, 1);
    assert_eq!(point.x, ((0x0Au16) << 4) | 0x05);
    assert_eq!(point.y, ((0x14u16) << 4) | 0x07);
    assert_eq!(point.area, 0x32);
    assert!(touches[1].is_none());

    i2c.done();
}

#[test]
fn get_attribute_populates_chip_info_on_success_async() {
    let expectations = attribute_expectations_success();
    let mut i2c = I2cMock::new(&expectations);

    let mut driver = CST328::new(i2c.clone(), NoopDelay::new());
    block_on(async { driver.get_attribute().await.unwrap() });

    let info = driver.chip_info();
    assert_eq!(info.fw_crc, 0xCACA);
    assert_eq!(info.project_id, 0x1234);
    assert_eq!(info.chip_id, 0x0328);
    assert_eq!(info.fw_build, 0x0102);
    assert_eq!(info.fw_minor, 0x05);
    assert_eq!(info.fw_major, 0x03);
    assert_eq!(info.resolution_x, 240);
    assert_eq!(info.resolution_y, 320);
    assert_eq!(driver.model_name(), "CST328/CST3530");

    i2c.done();
}

#[test]
fn get_attribute_rejects_bad_fw_crc_async() {
    let expectations = attribute_expectations(&CHECK_CODE_INVALID);
    let mut i2c = I2cMock::new(&expectations);

    let mut driver = CST328::new(i2c.clone(), NoopDelay::new());
    let result = block_on(async { driver.get_attribute().await });
    assert!(matches!(result, Err(Error::InvalidCheckCode)));

    i2c.done();
}

#[test]
fn set_mode_writes_normal_mode_register_async() {
    let expectations = [I2cTransaction::write(ADDR, REG_NORMAL_MODE_BYTES.to_vec())];
    let mut i2c = I2cMock::new(&expectations);

    let mut driver = CST328::new(i2c.clone(), NoopDelay::new());
    block_on(async { driver.set_mode(RunMode::Normal).await.unwrap() });

    i2c.done();
}

#[test]
fn set_mode_writes_debug_info_mode_register_async() {
    let expectations = [I2cTransaction::write(
        ADDR,
        REG_DEBUG_INFO_MODE_BYTES.to_vec(),
    )];
    let mut i2c = I2cMock::new(&expectations);

    let mut driver = CST328::new(i2c.clone(), NoopDelay::new());
    block_on(async { driver.set_mode(RunMode::DebugInfo).await.unwrap() });

    i2c.done();
}
