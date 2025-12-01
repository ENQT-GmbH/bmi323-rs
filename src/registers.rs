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
    /// Lower time Register
    pub const SENSOR_TIME_0: u8 = 0x0A;
    /// Upper time Register
    pub const SENSOR_TIME_1: u8 = 0x0B;
    ///amounts of bytes in FIFO
    pub const FIFO_FILL_LEVEL: u8 = 0x15;
    ///Data in FIFO, reads as 0x8000 when over read
    pub const FIFO_DATA: u8 = 0x16;
    /// Accelerometer configuration register address
    pub const ACC_CONF: u8 = 0x20;
    /// Gyroscope configuration register address
    pub const GYR_CONF: u8 = 0x21;
    /// FIFO watermark level in bytes Register
    pub const FIFO_WATERMARK: u8 = 0x35;
    /// FIFO configuration Register
    pub const FIFO_CONF: u8 = 0x36;
    /// FIFO control Register
    pub const FIFO_CTRL: u8 = 0x37;
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
    pub const INT_STATUS_INT1 : u8 = 0x0D;
    pub const INT_STATUS_INT2 : u8 = 0x0E;
    pub const INT_STATUS_INT_IBI : u8 = 0x0F;
    /// Expected chip ID for BMI323
    pub const BMI323_CHIP_ID: u8 = 0x43;
    /// Soft reset command value
    pub const CMD_SOFT_RESET: u16 = 0xDEAF;
}
