use crate::{
    interface::{I2cInterface, ReadData, SpiInterface, WriteData},
    types::get_sensor3d_data,
    AccelConfig, AccelerometerPowerMode, AccelerometerRange, AnyMotionConfig, Bmi323, Error,
    FifoConfig, FifoData, GyroConfig, GyroscopePowerMode, GyroscopeRange, IOInterruptConfig,
    InterruptLatch, InterruptMapConfig, InterruptPin, InterruptSource, MotionAxes, NoMotionConfig,
    Register, Sensor3DData, Sensor3DDataScaled, SensorType,
};
use embedded_hal::delay::DelayNs;

impl<I2C, D> Bmi323<I2cInterface<I2C>, D>
where
    D: DelayNs,
{
    /// Create a new BMI323 device instance
    ///
    /// # Arguments
    ///
    /// * `iface` - The communication interface
    /// * `delay` - A delay provider
    pub fn new_with_i2c(i2c: I2C, address: u8, delay: D) -> Self {
        Bmi323 {
            iface: I2cInterface { i2c, address },
            delay,
            accel_range: AccelerometerRange::default(),
            gyro_range: GyroscopeRange::default(),
            fifo_config: Default::default(),
            fifo_message_len: 0,
        }
    }
}

impl<SPI, D> Bmi323<SpiInterface<SPI>, D>
where
    D: DelayNs,
{
    /// Create a new BMI323 device instance
    ///
    /// # Arguments
    ///
    /// * `iface` - The communication interface
    /// * `delay` - A delay provider
    pub fn new_with_spi(spi: SPI, delay: D) -> Self {
        Bmi323 {
            iface: SpiInterface { spi },
            delay,
            accel_range: AccelerometerRange::default(),
            gyro_range: GyroscopeRange::default(),
            fifo_config: Default::default(),
            fifo_message_len: 0,
        }
    }
}

impl<DI, D, E> Bmi323<DI, D>
where
    DI: ReadData<Error = Error<E>> + WriteData<Error = Error<E>>,
    D: DelayNs,
{
    /// Initialize the device
    pub fn init(&mut self) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::CMD, Register::CMD_SOFT_RESET)?;
        self.delay.delay_us(2000);

        //let mut reg_data = [0u8; 3];
        //reg_data[0] = 0x01; // sensor error conditins register
        let status = self.read_register(0x01)?;
        if (status & 0b0000_0001) != 0 {
            return Err(Error::InvalidDevice);
        }

        let result = self.read_register(Register::CHIPID)?;
        if result != Register::BMI323_CHIP_ID {
            return Err(Error::InvalidDevice);
        }
        Ok(())
    }

    /// Enable the feature engine.
    ///
    /// This must be called after a reset. Once the engine is disabled, the
    /// device must be reset before it can be enabled again.
    pub fn enable_feature_engine(&mut self) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::FEATURE_IO2, Register::FEATURE_ENGINE_INIT)?;
        self.write_register_16bit(Register::FEATURE_IO_STATUS, Register::FEATURE_IO_SYNC)?;
        self.write_register_16bit(Register::FEATURE_CTRL, Register::FEATURE_ENGINE_ENABLED)?;

        for _ in 0..10 {
            self.delay.delay_ms(10);
            if self.read_register_16bit(Register::FEATURE_IO1)? & 1 != 0 {
                return Ok(());
            }
        }

        Err(Error::Timeout)
    }

    /// Disable the feature engine.
    ///
    /// A soft reset or power cycle is required before the feature engine can
    /// be enabled again.
    pub fn disable_feature_engine(&mut self) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::FEATURE_CTRL, Register::FEATURE_ENGINE_DISABLED)
    }

    /// Configure and enable any-motion detection on the selected axes.
    ///
    /// The accelerometer must already be configured and enabled.
    pub fn configure_any_motion(
        &mut self,
        config: AnyMotionConfig,
        axes: MotionAxes,
    ) -> Result<(), Error<E>> {
        self.set_any_motion_config(config)?;
        self.set_any_motion_axes(axes)
    }

    /// Configure and enable no-motion detection on the selected axes.
    ///
    /// The accelerometer must already be configured and enabled.
    pub fn configure_no_motion(
        &mut self,
        config: NoMotionConfig,
        axes: MotionAxes,
    ) -> Result<(), Error<E>> {
        self.set_no_motion_config(config)?;
        self.set_no_motion_axes(axes)
    }

    /// Set the feature engine's any-motion parameters.
    pub fn set_any_motion_config(&mut self, config: AnyMotionConfig) -> Result<(), Error<E>> {
        let words = NoMotionConfig {
            threshold: config.threshold,
            reference_update: config.reference_update,
            hysteresis: config.hysteresis,
            duration: config.duration,
            wait_time: config.wait_time,
        }
        .config_words()
        .ok_or(Error::InvalidConfig)?;
        self.write_feature_config(Register::ANY_MOTION_CONFIG_ADDR, words)
    }

    /// Set the feature engine's no-motion parameters.
    pub fn set_no_motion_config(&mut self, config: NoMotionConfig) -> Result<(), Error<E>> {
        let words = config.config_words().ok_or(Error::InvalidConfig)?;
        self.write_feature_config(Register::NO_MOTION_CONFIG_ADDR, words)
    }

    /// Enable any-motion detection on selected axes. Pass [`MotionAxes::none`]
    /// to disable it.
    pub fn set_any_motion_axes(&mut self, axes: MotionAxes) -> Result<(), Error<E>> {
        self.update_motion_axes(axes, 3)
    }

    /// Enable no-motion detection on selected axes. Pass [`MotionAxes::none`]
    /// to disable it.
    pub fn set_no_motion_axes(&mut self, axes: MotionAxes) -> Result<(), Error<E>> {
        self.update_motion_axes(axes, 0)
    }

    fn write_feature_config(&mut self, address: u16, words: [u16; 3]) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::FEATURE_DATA_ADDR, address)?;

        let first = words[0].to_le_bytes();
        let second = words[1].to_le_bytes();
        let third = words[2].to_le_bytes();
        self.iface.write_data(&[
            Register::FEATURE_DATA_TX,
            first[0],
            first[1],
            second[0],
            second[1],
            third[0],
            third[1],
        ])
    }

    fn update_motion_axes(&mut self, axes: MotionAxes, shift: u8) -> Result<(), Error<E>> {
        let current = self.read_register_16bit(Register::FEATURE_IO0)?;
        let mask = 0x07 << shift;
        let updated = (current & !mask) | (axes.bits() << shift);

        // FEATURE_IO0 must be cleared before changing an active configuration.
        self.write_register_16bit(Register::FEATURE_IO0, 0)?;
        self.write_register_16bit(Register::FEATURE_IO0, updated)?;
        self.write_register_16bit(Register::FEATURE_IO_STATUS, Register::FEATURE_IO_SYNC)
    }

    /// Set the accelerometer configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The accelerometer configuration
    pub fn set_accel_config(&mut self, config: AccelConfig) -> Result<(), Error<E>> {
        let reg_data = self.config_to_reg_data(config);
        self.write_register_16bit(Register::ACC_CONF, reg_data)?;
        self.accel_range = config.range;

        // Wait for accelerometer data to be ready
        if config.mode != AccelerometerPowerMode::Disable {
            self.wait_for_data_ready(SensorType::Accelerometer)?;
        }

        Ok(())
    }

    /// Set the gyroscope configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The gyroscope configuration
    pub fn set_gyro_config(&mut self, config: GyroConfig) -> Result<(), Error<E>> {
        let reg_data = self.config_to_reg_data(config);
        self.write_register_16bit(Register::GYR_CONF, reg_data)?;
        self.gyro_range = config.range;

        // Wait for gyroscope data to be ready
        if config.mode != GyroscopePowerMode::Disable {
            self.wait_for_data_ready(SensorType::Gyroscope)?;
        }
        Ok(())
    }

    /// Set interrupt register configuration
    /// Note: Interrupt pins are disabled by default use set_io_interrupt_config to configure them
    ///
    /// # Arguments
    ///
    /// * `config` - The interruptMap configuration
    pub fn set_interrupt_mapping_config(
        &mut self,
        config: InterruptMapConfig,
    ) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::INT_MAP1, config.map1())?;
        self.write_register_16bit(Register::INT_MAP2, config.map2())
    }

    /// Set IO interrupt register configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The IOinterrupt configuration
    pub fn set_io_interrupt_config(&mut self, config: IOInterruptConfig) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::INT_CTRL, u16::from(config))
    }

    /// Set latching interrupt register configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The interrupt latching configuration
    pub fn set_interrupt_lachting_config(
        &mut self,
        config: InterruptLatch,
    ) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::INT_CTRL, config as u16)
    }

    fn config_to_reg_data<T>(&self, config: T) -> u16
    where
        T: Into<u16> + Copy,
    {
        let config: u16 = config.into();
        config
    }

    fn read_sensor_data(&mut self, sensor_type: SensorType) -> Result<Sensor3DData, Error<E>> {
        let (base_reg, data_size) = match sensor_type {
            SensorType::Accelerometer => (Register::ACC_DATA_X, 1 + 6),
            SensorType::Gyroscope => (Register::GYR_DATA_X, 1 + 6),
        };

        let mut data = [0u8; 21]; // Use the larger size
        data[0] = base_reg;
        let sensor_data = self.read_data(&mut data[0..data_size])?;

        Ok(Sensor3DData {
            x: i16::from_le_bytes([sensor_data[0], sensor_data[1]]),
            y: i16::from_le_bytes([sensor_data[2], sensor_data[3]]),
            z: i16::from_le_bytes([sensor_data[4], sensor_data[5]]),
        })
    }

    /// Read the LSB for the accelerometer
    pub fn read_accel_data(&mut self) -> Result<Sensor3DData, Error<E>> {
        self.read_sensor_data(SensorType::Accelerometer)
    }

    /// Read the LSB for the gyroscope
    pub fn read_gyro_data(&mut self) -> Result<Sensor3DData, Error<E>> {
        self.read_sensor_data(SensorType::Gyroscope)
    }

    /// Read the LSB for the accelerometer and return the scaled value as mps2
    pub fn read_accel_data_scaled(&mut self) -> Result<Sensor3DDataScaled, Error<E>> {
        let raw_data = self.read_accel_data()?;
        Ok(raw_data.to_mps2(self.accel_range.to_g())) // Assuming 16-bit width
    }

    /// Read the LSB for the gyroscope and return the scaled value as dps
    pub fn read_gyro_data_scaled(&mut self) -> Result<Sensor3DDataScaled, Error<E>> {
        let raw_data = self.read_gyro_data()?;
        Ok(raw_data.to_dps(self.gyro_range.to_dps())) // Assuming 16-bit width
    }

    fn write_register_16bit(&mut self, reg: u8, value: u16) -> Result<(), Error<E>> {
        let bytes = value.to_le_bytes();
        self.iface.write_data(&[reg, bytes[0], bytes[1]])
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, Error<E>> {
        self.iface.read_register(reg)
    }

    fn read_register_16bit(&mut self, reg: u8) -> Result<u16, Error<E>> {
        let mut data = [reg, 0, 0];
        let data = self.read_data(&mut data)?;
        Ok(u16::from_le_bytes([data[0], data[1]]))
    }

    fn read_data<'a>(&mut self, data: &'a mut [u8]) -> Result<&'a [u8], Error<E>> {
        self.iface.read_data(data)
    }

    pub fn wait_for_data_ready(&mut self, sensor_type: SensorType) -> Result<(), Error<E>> {
        const MAX_RETRIES: u16 = 1200;
        let mut retries = 0;

        while !self.is_data_ready(sensor_type)? {
            if retries >= MAX_RETRIES {
                return Err(Error::Timeout);
            }
            self.delay.delay_ms(1);
            retries += 1;
        }

        Ok(())
    }

    fn is_data_ready(&mut self, sensor_type: SensorType) -> Result<bool, Error<E>> {
        let status = self.read_register(Register::STATUS)?;
        match sensor_type {
            SensorType::Accelerometer => Ok((status & 0b1000_0000) != 0), // Check bit 7 (drdy_acc)
            SensorType::Gyroscope => Ok((status & 0b0100_0000) != 0),     // Check bit 6 (drdy_gyr)
        }
    }

    /// Read the Timestamp from the device
    pub fn read_sensor_timestamp(&mut self) -> Result<u32, Error<E>> {
        let mut data = [Register::SENSOR_TIME_0, 0u8, 0u8, 0u8, 0u8];
        self.read_data(&mut data)?;
        Ok(u32::from_le_bytes([data[1], data[2], data[3], data[4]]))
    }

    ///configures the FIFO
    pub fn set_fifo_config(&mut self, config: &FifoConfig) -> Result<(), Error<E>> {
        if *config == self.fifo_config {
            return Ok(());
        }
        self.write_register_16bit(Register::FIFO_CONF, config.to_register_value())?;
        if let Some(watermark) = config.watermark_level {
            // convert number of messages to number of 16 bit words while keeping to max value
            let watermark = (watermark * (config.fifo_message_len()) as u16).min(0x3FF);
            self.write_register_16bit(Register::FIFO_WATERMARK, watermark)?;
        }
        self.fifo_config = *config;
        self.fifo_message_len = config.fifo_message_len();
        Ok(())
    }

    ///flushes the FIFO
    pub fn flush_fifo(&mut self) -> Result<(), Error<E>> {
        self.write_register_16bit(Register::FIFO_CTRL, 0x01)
    }

    ///reads the number of words in the fifo
    fn get_fifo_fill_state(&mut self) -> Result<u16, Error<E>> {
        let mut data = [Register::FIFO_FILL_LEVEL, 0, 0];
        let res = self.read_data(&mut data)?;
        Ok(u16::from_le_bytes([res[0], res[1]]))
    }

    /// reads the number of entries in the FIFO
    pub fn get_fifo_entry_count(&mut self) -> Result<u16, Error<E>> {
        Ok(self.get_fifo_fill_state()? / self.fifo_message_len)
    }

    /// reads one entry from the Fifo
    pub fn read_fifo_entry(&mut self) -> Result<FifoData, Error<E>> {
        const FIFO_MESSAGE_LEN_MAX: usize = 22;
        let message_len = self.fifo_message_len;
        if message_len > self.get_fifo_fill_state()? {
            return Err(Error::FifoEmpty);
        }
        let mut buffer = [0u8; FIFO_MESSAGE_LEN_MAX + 1];
        buffer[0] = Register::FIFO_DATA;
        let fifo_data = self.read_data(&mut buffer[..(message_len as usize * 2) + 1])?;
        let mut index = 0;
        let mut ret = FifoData::default();
        if self.fifo_config.accel_enabled {
            let sensor_data = get_sensor3d_data(&fifo_data[index..]);
            // fill data for invalid data
            if sensor_data.x != 0x7f01 {
                ret.accel = Some(sensor_data.to_mps2(self.accel_range.to_g()));
            }
            index += 6;
        }
        if self.fifo_config.gyro_enabled {
            let sensor_data = get_sensor3d_data(&fifo_data[index..]);
            if sensor_data.x != 0x7f02 {
                ret.gyro =
                    Some(get_sensor3d_data(&fifo_data[index..]).to_mps2(self.accel_range.to_g()));
            }
            index += 6;
        }
        if self.fifo_config.temp_enabled {
            ret.temp = Some(u16::from_le_bytes([fifo_data[index], fifo_data[index + 1]]));
            index += 2
        }
        if self.fifo_config.timestamp_enabled {
            ret.timestamp = Some(u16::from_le_bytes([fifo_data[index], fifo_data[index + 1]]));
        }
        Ok(ret)
    }

    /// reads the interrupt source for the given pin. This resets the interrupt.
    pub fn get_int_status(&mut self, pin: InterruptPin) -> Result<InterruptSource, Error<E>> {
        let reg = match pin {
            InterruptPin::Int1 => Register::INT_STATUS_INT1,
            InterruptPin::Int2 => Register::INT_STATUS_INT2,
            InterruptPin::IC3IBI => Register::INT_STATUS_INT_IBI,
        };
        let mut data = [reg, 0, 0];
        let res = self.read_data(&mut data)?;
        Ok(InterruptSource::from(u16::from_le_bytes([res[0], res[1]])))
    }

    #[cfg(feature = "debug")]
    pub fn debug_read(&mut self, register: u8) -> Result<u16, Error<E>> {
        let mut data = [register, 0, 0];
        let res = self.read_data(&mut data)?;
        Ok(u16::from_le_bytes([res[0], res[1]]))
    }
    #[cfg(feature = "debug")]
    pub fn debug_write(&mut self, register: u8, value: u16) -> Result<(), Error<E>> {
        self.write_register_16bit(register, value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_sensor3d_data(data: &[u8]) -> Sensor3DData {
        Sensor3DData {
            x: i16::from_le_bytes([data[0], data[1]]),
            y: i16::from_le_bytes([data[2], data[3]]),
            z: i16::from_le_bytes([data[4], data[5]]),
        }
    }
    mod sensor3d_data {
        use super::*;

        #[test]
        fn can_decode_positive_array() {
            let result = get_sensor3d_data(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
            assert_eq!(
                result,
                Sensor3DData {
                    x: 0x0201,
                    y: 0x0403,
                    z: 0x0605
                }
            );
        }

        #[test]
        fn can_decode_negative_array() {
            let result = get_sensor3d_data(&[0x0B, 0x86, 0x0B, 0x86, 0x0B, 0x86]);
            assert_eq!(
                result,
                Sensor3DData {
                    x: -31221,
                    y: -31221,
                    z: -31221
                }
            );
        }
    }
}
