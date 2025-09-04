#![no_std]

/// BMI323 driver for Rust
///
/// This module provides a high-level interface for interacting with the Bosch BMI323 IMU.
/// It supports both I2C and SPI interfaces and allows for configuration of accelerometer
/// and gyroscope settings.
pub mod device;
pub mod interface;
mod registers;
pub use registers::Register;
mod types;
pub use types::{
    AccelerometerPowerMode, AccelerometerRange, AverageNum, Bandwidth, Error, GyroscopePowerMode,
    GyroscopeRange, OutputDataRate, Sensor3DData, Sensor3DDataScaled, 
    InterruptEnable, InterruptLatch, InterruptLevel, InterruptMapping, InterruptOd,
};
mod sensor_data;
pub use sensor_data::*;

/// Main struct representing the BMI323 device
pub struct Bmi323<DI, D> {
    /// Communication interface (I2C or SPI)
    iface: DI,
    /// Delay provider
    delay: D,
    /// Current accelerometer range
    accel_range: AccelerometerRange,
    /// Current gyroscope range
    gyro_range: GyroscopeRange,
}

/// Configuration for the accelerometer
#[derive(Debug, Clone, Copy)]
pub struct AccelConfig {
    /// Output data rate
    pub odr: OutputDataRate,
    /// Measurement range
    pub range: AccelerometerRange,
    /// Bandwidth
    pub bw: Bandwidth,
    /// Number of samples to average
    pub avg_num: AverageNum,
    /// Power mode
    pub mode: AccelerometerPowerMode,
}

impl AccelConfig {
    /// Create a new AccelConfigBuilder
    pub fn builder() -> AccelConfigBuilder {
        AccelConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AccelConfigBuilder {
    odr: Option<OutputDataRate>,
    range: Option<AccelerometerRange>,
    bw: Option<Bandwidth>,
    avg_num: Option<AverageNum>,
    mode: Option<AccelerometerPowerMode>,
}

/// Builder for AccelConfig
impl Default for AccelConfigBuilder {
    fn default() -> Self {
        Self {
            odr: None,
            range: None,
            bw: None,
            avg_num: None,
            mode: None,
        }
    }
}

impl AccelConfigBuilder {
    /// Set the output data rate
    pub fn odr(mut self, odr: OutputDataRate) -> Self {
        self.odr = Some(odr);
        self
    }

    /// Set the measurement range
    pub fn range(mut self, range: AccelerometerRange) -> Self {
        self.range = Some(range);
        self
    }

    /// Set the bandwidth
    pub fn bw(mut self, bw: Bandwidth) -> Self {
        self.bw = Some(bw);
        self
    }

    /// Set the number of samples to average
    pub fn avg_num(mut self, avg_num: AverageNum) -> Self {
        self.avg_num = Some(avg_num);
        self
    }

    /// Set the power mode
    pub fn mode(mut self, mode: AccelerometerPowerMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Build the AccelConfig
    pub fn build(self) -> AccelConfig {
        AccelConfig {
            odr: self.odr.unwrap_or(OutputDataRate::Odr100hz),
            range: self.range.unwrap_or(AccelerometerRange::G8),
            bw: self.bw.unwrap_or(Bandwidth::OdrQuarter),
            avg_num: self.avg_num.unwrap_or(AverageNum::Avg1),
            mode: self.mode.unwrap_or(AccelerometerPowerMode::Normal),
        }
    }
}

/// Configuration for the gyroscope
#[derive(Debug, Clone, Copy)]
pub struct GyroConfig {
    /// Output data rate
    pub odr: OutputDataRate,
    /// Measurement range
    pub range: GyroscopeRange,
    /// Bandwidth
    pub bw: Bandwidth,
    /// Number of samples to average
    pub avg_num: AverageNum,
    /// Power mode
    pub mode: GyroscopePowerMode,
}

impl GyroConfig {
    /// Create a new GyroConfigBuilder
    pub fn builder() -> GyroConfigBuilder {
        GyroConfigBuilder::default()
    }
}

/// Builder for GyroConfig
#[derive(Debug, Clone, Copy)]
pub struct GyroConfigBuilder {
    odr: Option<OutputDataRate>,
    range: Option<GyroscopeRange>,
    bw: Option<Bandwidth>,
    avg_num: Option<AverageNum>,
    mode: Option<GyroscopePowerMode>,
}

impl Default for GyroConfigBuilder {
    fn default() -> Self {
        Self {
            odr: None,
            range: None,
            bw: None,
            avg_num: None,
            mode: None,
        }
    }
}

impl GyroConfigBuilder {
    /// Set the output data rate
    pub fn odr(mut self, odr: OutputDataRate) -> Self {
        self.odr = Some(odr);
        self
    }

    /// Set the measurement range
    pub fn range(mut self, range: GyroscopeRange) -> Self {
        self.range = Some(range);
        self
    }

    /// Set the bandwidth
    pub fn bw(mut self, bw: Bandwidth) -> Self {
        self.bw = Some(bw);
        self
    }

    /// Set the power mode
    pub fn avg_num(mut self, avg_num: AverageNum) -> Self {
        self.avg_num = Some(avg_num);
        self
    }

    /// Set the power mode
    pub fn mode(mut self, mode: GyroscopePowerMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Build the GyroConfig
    pub fn build(self) -> GyroConfig {
        GyroConfig {
            odr: self.odr.unwrap_or(OutputDataRate::Odr100hz),
            range: self.range.unwrap_or(GyroscopeRange::DPS2000),
            bw: self.bw.unwrap_or(Bandwidth::OdrHalf),
            avg_num: self.avg_num.unwrap_or(AverageNum::Avg1),
            mode: self.mode.unwrap_or(GyroscopePowerMode::Normal),
        }
    }
}

impl From<AccelConfig> for u16 {
    /// Convert AccelConfig to a 16-bit register value
    fn from(config: AccelConfig) -> Self {
        (config.odr as u16 & 0x0F)
            | ((config.range as u16 & 0x07) << 4)
            | ((config.bw as u16 & 0x01) << 7)
            | ((config.avg_num as u16 & 0x07) << 8)
            | ((config.mode as u16 & 0x07) << 12)
    }
}

impl From<GyroConfig> for u16 {
    /// Convert GyroConfig to a 16-bit register value
    fn from(config: GyroConfig) -> Self {
        (config.odr as u16 & 0x0F)
            | ((config.range as u16 & 0x07) << 4)
            | ((config.bw as u16 & 0x01) << 7)
            | ((config.avg_num as u16 & 0x07) << 8)
            | ((config.mode as u16 & 0x07) << 12)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InterruptMapConfig{
    no_motion : InterruptMapping,
    any_motion : InterruptMapping,
    flat : InterruptMapping,
    orientation : InterruptMapping,
    step_detector : InterruptMapping,
    step_counter : InterruptMapping,
    sig_motion : InterruptMapping,
    tilt_out : InterruptMapping,
    tap : InterruptMapping,
    i3c : InterruptMapping,
    err_status : InterruptMapping,
    temp_drdy : InterruptMapping,
    gyr_drdy : InterruptMapping,
    acc_drdy : InterruptMapping,
    fifo_watermark : InterruptMapping,
    fifo_full : InterruptMapping,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InterruptMapConfigBuilder{
    no_motion : Option<InterruptMapping>,
    any_motion : Option<InterruptMapping>,
    flat : Option<InterruptMapping>,
    orientation : Option<InterruptMapping>,
    step_detector : Option<InterruptMapping>,
    step_counter : Option<InterruptMapping>,
    sig_motion : Option<InterruptMapping>,
    tilt_out : Option<InterruptMapping>,
    tap : Option<InterruptMapping>,
    i3c : Option<InterruptMapping>,
    err_status : Option<InterruptMapping>,
    temp_drdy : Option<InterruptMapping>,
    gyr_drdy : Option<InterruptMapping>,
    acc_drdy : Option<InterruptMapping>,
    fifo_watermark : Option<InterruptMapping>,
    fifo_full : Option<InterruptMapping>,
}

impl InterruptMapConfigBuilder {
    pub fn no_motion(mut self, mapping: InterruptMapping)->Self{
        self.no_motion = Some(mapping);
        self
    }
    pub fn any_motion(mut self, mapping: InterruptMapping)->Self{
        self.any_motion = Some(mapping);
        self
    }
    pub fn flat(mut self, mapping: InterruptMapping)->Self{
        self.flat = Some(mapping);
        self
    }
    pub fn orientation(mut self, mapping: InterruptMapping)->Self{
        self.orientation = Some(mapping);
        self
    }
    pub fn step_detector(mut self, mapping: InterruptMapping)->Self{
        self.step_detector = Some(mapping);
        self
    }
    pub fn step_counter(mut self, mapping: InterruptMapping)->Self{
        self.step_counter = Some(mapping);
        self
    }
    pub fn sig_motion(mut self, mapping: InterruptMapping)->Self{
        self.sig_motion = Some(mapping);
        self
    }
    pub fn tilt_out(mut self, mapping: InterruptMapping)->Self{
        self.tilt_out = Some(mapping);
        self
    }
    pub fn tap(mut self, mapping: InterruptMapping)->Self{
        self.tap = Some(mapping);
        self
    }
    pub fn i3c(mut self, mapping: InterruptMapping)->Self{
        self.i3c = Some(mapping);
        self
    }
    pub fn err_status(mut self, mapping: InterruptMapping)->Self{
        self.err_status = Some(mapping);
        self
    }
    pub fn temp_drdy(mut self, mapping: InterruptMapping)->Self{
        self.temp_drdy = Some(mapping);
        self
    }
    pub fn gyr_drdy(mut self, mapping: InterruptMapping)->Self{
        self.gyr_drdy = Some(mapping);
        self
    }
    pub fn acc_drdy(mut self, mapping: InterruptMapping)->Self{
        self.acc_drdy = Some(mapping);
        self
    }
    pub fn fifo_watermark(mut self, mapping: InterruptMapping)->Self{
        self.fifo_watermark = Some(mapping);
        self
    }
    pub fn fifo_full(mut self, mapping: InterruptMapping)->Self{
        self.fifo_full = Some(mapping);
        self
    }
    pub fn build(self)->InterruptMapConfig{
        InterruptMapConfig {
            no_motion: self.no_motion.unwrap_or(InterruptMapping::Disabled),
            any_motion : self.any_motion.unwrap_or(InterruptMapping::Disabled),
            flat : self.flat.unwrap_or(InterruptMapping::Disabled),
            orientation : self.orientation.unwrap_or(InterruptMapping::Disabled),
            step_detector : self.step_detector.unwrap_or(InterruptMapping::Disabled),
            step_counter : self.step_counter.unwrap_or(InterruptMapping::Disabled),
            sig_motion  : self.sig_motion.unwrap_or(InterruptMapping::Disabled),
            tilt_out : self.tilt_out.unwrap_or(InterruptMapping::Disabled),
            tap: self.tap.unwrap_or(InterruptMapping::Disabled),
            i3c : self.i3c.unwrap_or(InterruptMapping::Disabled),
            err_status : self.err_status.unwrap_or(InterruptMapping::Disabled),
            temp_drdy : self.temp_drdy.unwrap_or(InterruptMapping::Disabled),
            gyr_drdy : self.gyr_drdy.unwrap_or(InterruptMapping::Disabled),
            acc_drdy : self.acc_drdy.unwrap_or(InterruptMapping::Disabled),
            fifo_watermark  : self.fifo_watermark.unwrap_or(InterruptMapping::Disabled),
            fifo_full : self.fifo_full.unwrap_or(InterruptMapping::Disabled),
        }
    }
}

impl InterruptMapConfig {
    pub fn builder() -> InterruptMapConfigBuilder{
        InterruptMapConfigBuilder::default()
    }
    fn map1(&self) -> u16 {
        self.no_motion as u16  & 0x04 | 
        (self.any_motion as u16  & 0x04) << 2 | 
        (self.flat as u16 & 0x04) << 4 | 
        (self.orientation as u16 & 0x04) << 6 | 
        (self.step_detector as u16 & 0x04) << 8 | 
        (self.step_counter as u16 & 0x04) << 10 | 
        (self.sig_motion as u16 & 0x04) << 12 | 
        (self.tilt_out as u16 & 0x04) << 14 
    }

    fn map2(&self) -> u16 {
        self.tap as u16  & 0x04 | 
        (self.i3c as u16  & 0x04) << 2 | 
        (self.err_status as u16 & 0x04) << 4 | 
        (self.temp_drdy as u16 & 0x04) << 6 | 
        (self.gyr_drdy as u16 & 0x04) << 8 | 
        (self.acc_drdy as u16 & 0x04) << 10 | 
        (self.fifo_watermark as u16 & 0x04) << 12 | 
        (self.fifo_full as u16 & 0x04) << 14 
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IOInterruptConfig{
    int1_lvl : InterruptLevel,
    int1_od : InterruptOd,
    int1_en : InterruptEnable,
    int2_lvl : InterruptLevel,
    int2_od : InterruptOd,
    int2_en : InterruptEnable,
}

impl IOInterruptConfig{
    pub fn builder() -> IOInterruptConfigBuilder{
        IOInterruptConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IOInterruptConfigBuilder{
    int1_lvl : Option<InterruptLevel>,
    int1_od : Option<InterruptOd>,
    int1_en : Option<InterruptEnable>,
    int2_lvl : Option<InterruptLevel>,
    int2_od : Option<InterruptOd>,
    int2_en : Option<InterruptEnable>,
}

impl IOInterruptConfigBuilder{
    pub fn int1_lvl(mut self, level: InterruptLevel)->Self{
        self.int1_lvl = Some(level);
        self
    }
    pub fn int1_od(mut self, od: InterruptOd)->Self{
        self.int1_od = Some(od);
        self
    }
    pub fn int1_enable(mut self, enable: InterruptEnable)->Self{
        self.int1_en = Some(enable);
        self
    }
    pub fn int2_lvl(mut self, level: InterruptLevel)->Self{
        self.int2_lvl = Some(level);
        self
    }
    pub fn int2_od(mut self, od: InterruptOd)->Self{
        self.int1_od = Some(od);
        self
    }
    pub fn int2_enable(mut self, enable: InterruptEnable)->Self{
        self.int2_en = Some(enable);
        self
    }
    pub fn build(self)->IOInterruptConfig{
        IOInterruptConfig {
            int1_lvl: self.int1_lvl.unwrap_or(InterruptLevel::ActiveLow),
            int1_od: self.int1_od.unwrap_or(InterruptOd::PushPull),
            int1_en : self.int1_en.unwrap_or(InterruptEnable::Disabled),
            int2_lvl: self.int2_lvl.unwrap_or(InterruptLevel::ActiveLow),
            int2_od: self.int2_od.unwrap_or(InterruptOd::PushPull),
            int2_en : self.int2_en.unwrap_or(InterruptEnable::Disabled),

        }
    }
}

impl From<IOInterruptConfig> for u16 {
    fn from(config: IOInterruptConfig) -> Self {
        (config.int1_lvl as u16 & 0x1) |
        (config.int1_od as u16 & 0x1) << 1 |
        (config.int1_en as u16 & 0x1) << 2 |
        (config.int2_lvl as u16 & 0x1) << 7 |
        (config.int2_od as u16 & 0x1) << 8 |
        (config.int2_en as u16 & 0x1) << 9
    }
}