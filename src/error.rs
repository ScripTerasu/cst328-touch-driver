/// Errors emitted by the CST328/CST3530 driver.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum Error<E> {
    /// `get_attribute()` read a firmware CRC (at [`crate::registers::REG_CHECK_CODE`])
    /// that didn't match the expected `0xCACA`, indicating a garbled or
    /// unsupported attribute read.
    InvalidCheckCode,
    /// A low-level I2C error.
    I2C(E),
}
