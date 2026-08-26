use bmi323::{
    AccelConfig, AccelerometerPowerMode, AccelerometerRange, AnyMotionConfig, AverageNum,
    Bandwidth, Bmi323, GyroConfig, GyroscopePowerMode, GyroscopeRange, MotionAxes, NoMotionConfig,
    OutputDataRate,
};
use embedded_hal_mock::eh1::delay::NoopDelay as MockDelay;
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

#[test]
fn test_conversion() {
    let accel_config = AccelConfig::builder()
        .odr(OutputDataRate::Odr100hz)
        .range(AccelerometerRange::G16)
        .bw(Bandwidth::OdrQuarter) // ODR/4
        .avg_num(AverageNum::Avg64)
        .mode(AccelerometerPowerMode::Normal)
        .build();
    assert_eq!(
        accel_config,
        AccelConfig::from(u16::from(accel_config.clone()))
    );

    let gyro_config = GyroConfig::builder()
        .odr(OutputDataRate::Odr100hz)
        .range(GyroscopeRange::DPS2000)
        .bw(Bandwidth::OdrQuarter)
        .avg_num(AverageNum::Avg64)
        .mode(GyroscopePowerMode::Normal)
        .build();
    assert_eq!(
        gyro_config,
        GyroConfig::from(u16::from(gyro_config.clone()))
    );
}

//I2C has two dummy bytes at the start of read
#[test]
fn test_bmi323_init() {
    let expectations = [
        I2cTransaction::write(0x68, vec![0x7E, 0xAF, 0xDE]), //soft reset
        I2cTransaction::write_read(0x68, vec![0x01], vec![0x00, 0x00, 0x00]), //state after reset
        I2cTransaction::write_read(0x68, vec![0x00], vec![0x00, 0x00, 0x43]), //chip ID
    ];

    let mut i2c = I2cMock::new(&expectations);
    let delay = MockDelay::new();
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, delay);

    bmi323.init().unwrap();

    i2c.done();
}

#[test]
fn test_disable_feature_engine() {
    let expectations = [I2cTransaction::write(0x68, vec![0x40, 0x00, 0x00])];
    let mut i2c = I2cMock::new(&expectations);
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, MockDelay::new());

    bmi323.disable_feature_engine().unwrap();

    i2c.done();
}

#[test]
fn test_configure_any_motion() {
    let expectations = [
        I2cTransaction::write(0x68, vec![0x41, 0x05, 0x00]),
        I2cTransaction::write(0x68, vec![0x42, 0x08, 0x10, 0x05, 0x00, 0xFA, 0xA0]),
        I2cTransaction::write_read(0x68, vec![0x10], vec![0x00, 0x00, 0x00, 0x00]),
        I2cTransaction::write(0x68, vec![0x10, 0x00, 0x00]),
        I2cTransaction::write(0x68, vec![0x10, 0x38, 0x00]),
        I2cTransaction::write(0x68, vec![0x14, 0x01, 0x00]),
    ];

    let mut i2c = I2cMock::new(&expectations);
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, MockDelay::new());

    bmi323
        .configure_any_motion(AnyMotionConfig::default(), MotionAxes::all())
        .unwrap();

    i2c.done();
}

#[test]
fn test_configure_no_motion_preserves_other_features() {
    let expectations = [
        I2cTransaction::write(0x68, vec![0x41, 0x08, 0x00]),
        I2cTransaction::write(0x68, vec![0x42, 0x1E, 0x10, 0x03, 0x00, 0xFA, 0xA0]),
        I2cTransaction::write_read(0x68, vec![0x10], vec![0x00, 0x00, 0x38, 0x00]),
        I2cTransaction::write(0x68, vec![0x10, 0x00, 0x00]),
        I2cTransaction::write(0x68, vec![0x10, 0x3F, 0x00]),
        I2cTransaction::write(0x68, vec![0x14, 0x01, 0x00]),
    ];

    let mut i2c = I2cMock::new(&expectations);
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, MockDelay::new());

    bmi323
        .configure_no_motion(NoMotionConfig::default(), MotionAxes::all())
        .unwrap();

    i2c.done();
}

#[test]
fn test_motion_config_rejects_values_that_do_not_fit() {
    let expectations = [];
    let mut i2c = I2cMock::new(&expectations);
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, MockDelay::new());
    let config = AnyMotionConfig {
        threshold: 4096,
        ..AnyMotionConfig::default()
    };

    assert!(matches!(
        bmi323.set_any_motion_config(config),
        Err(bmi323::Error::InvalidConfig)
    ));

    i2c.done();
}

#[test]
fn test_bmi323_set_sensor_config() {
    let expectations = [
        I2cTransaction::write(0x68, vec![0x20, 0xB8, 0x46]), // Accelerometer config
        I2cTransaction::write_read(0x68, vec![0x02], vec![0x00, 0x00, 0x80]), //check for data ready
        I2cTransaction::write(0x68, vec![0x21, 0x48, 0x46]), // Gyroscope config
        I2cTransaction::write_read(0x68, vec![0x02], vec![0x00, 0x00, 0x40]), //check for data ready
    ];

    let mut i2c = I2cMock::new(&expectations);
    let delay = MockDelay::new();
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, delay);

    let accel_config = AccelConfig::builder()
        .odr(OutputDataRate::Odr100hz)
        .range(AccelerometerRange::G16)
        .bw(Bandwidth::OdrQuarter) // ODR/4
        .avg_num(AverageNum::Avg64)
        .mode(AccelerometerPowerMode::Normal)
        .build();

    let gyro_config = GyroConfig::builder()
        .odr(OutputDataRate::Odr100hz)
        .range(GyroscopeRange::DPS2000)
        .bw(Bandwidth::OdrHalf) // ODR/2
        .avg_num(AverageNum::Avg64)
        .mode(GyroscopePowerMode::Normal)
        .build();

    bmi323.set_accel_config(accel_config).unwrap();
    bmi323.set_gyro_config(gyro_config).unwrap();

    i2c.done();
}

#[test]
fn test_bmi323_read_sensor_data() {
    let expectations = [I2cTransaction::write_read(
        0x68,
        vec![0x03],
        vec![0x00, 0x00, 0, 0, 0, 0, 0, 0],
    )];

    let mut i2c = I2cMock::new(&expectations);
    let delay = MockDelay::new();
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, delay);

    let sensor_data = bmi323.read_accel_data().unwrap();
    assert_eq!(sensor_data.x, 0);
    assert_eq!(sensor_data.y, 0);
    assert_eq!(sensor_data.z, 0);

    i2c.done();
}
#[test]
fn test_bmi323_read_int_source() {
    let expectations = [I2cTransaction::write_read(
        0x68,
        vec![0x0D],
        vec![0x00, 0x00, 0x00, 0x30],
    )];

    let mut i2c = I2cMock::new(&expectations);
    let delay = MockDelay::new();
    let mut bmi323 = Bmi323::new_with_i2c(i2c.clone(), 0x68, delay);

    let source = bmi323.get_int_status(bmi323::InterruptPin::Int1).unwrap();
    assert!(source.gyr_drdy);
    assert!(source.acc_drdy);
    assert!(!source.fifo_full);
    assert!(!source.fifo_watermark);

    i2c.done();
}
