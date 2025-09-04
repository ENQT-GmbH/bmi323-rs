/// BMI323 register addresses and constant values
pub struct Register;
impl Register {
    /// Chip ID register address
    pub const CHIPID: u8 = 0x00;
    /// Error register address
    pub const ERR_REG: u8 = 0x01;
    /// Status register address
    pub const STATUS: u8 = 0x02;
    /// Accelerometer X-axis data register address
    pub const ACC_DATA_X: u8 = 0x03;
    /// Gyroscope X-axis data register address
    pub const GYR_DATA_X: u8 = 0x06;
    /// Accelerometer configuration register address
    pub const ACC_CONF: u8 = 0x20;
    /// Gyroscope configuration register address
    pub const GYR_CONF: u8 = 0x21;
    /// Command register address
    pub const CMD: u8 = 0x7E;
    /// IO interrupt control register address
    pub const INT_CTRL: u8 = 0x38;
    /// Interrupt configuration register address
    pub const INT_CONF: u8 = 0x39;
    /// Interrupt Map 1 register address
    pub const INT_MAP1: u8 = 0x3A;
    /// Interrupt Map 2 register address
    pub const INT_MAP2: u8 = 0x3B;
    /// Expected chip ID for BMI323
    pub const BMI323_CHIP_ID: u8 = 0x43;
    /// Soft reset command value
    pub const CMD_SOFT_RESET: u16 = 0xDEAF;
}
