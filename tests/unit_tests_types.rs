use bmi323::{
    AccelConfig, AccelerometerRange, FifoConfig, GyroConfig, GyroscopeRange,
    InterruptMapConfigBuilder, InterruptSource,
};

#[test]
fn test_accelerometer_range_to_g() {
    assert_eq!(AccelerometerRange::G2.to_g(), 2.0);
    assert_eq!(AccelerometerRange::G4.to_g(), 4.0);
    assert_eq!(AccelerometerRange::G8.to_g(), 8.0);
    assert_eq!(AccelerometerRange::G16.to_g(), 16.0);
}

#[test]
fn test_gyroscope_range_to_dps() {
    assert_eq!(GyroscopeRange::DPS125.to_dps(), 125.0);
    assert_eq!(GyroscopeRange::DPS250.to_dps(), 250.0);
    assert_eq!(GyroscopeRange::DPS500.to_dps(), 500.0);
    assert_eq!(GyroscopeRange::DPS1000.to_dps(), 1000.0);
    assert_eq!(GyroscopeRange::DPS2000.to_dps(), 2000.0);
}

#[test]
fn test_accelerometer_range_default() {
    assert_eq!(AccelerometerRange::default(), AccelerometerRange::G8);
}

#[test]
fn test_gyroscope_range_default() {
    assert_eq!(GyroscopeRange::default(), GyroscopeRange::DPS2000);
}

#[test]
fn test_accelerometer_default() {
    println!("{:x?}", u16::from(AccelConfig::default()));
    assert_eq!(u16::from(AccelConfig::default()), 0x4028);
}

#[test]
fn test_gyroscope_default() {
    println!("{:x?}", u16::from(GyroConfig::default()));
    assert_eq!(u16::from(GyroConfig::default()), 0x4048);
}

#[test]
fn test_fifo_message_len() {
    let mut config = FifoConfig::default();
    assert_eq!(config.fifo_message_len(), 0);
    config.accel_enabled = true;
    assert_eq!(config.fifo_message_len(), 3);
    config.gyro_enabled = true;
    assert_eq!(config.fifo_message_len(), 6);
    config.timestamp_enabled = true;
    assert_eq!(config.fifo_message_len(), 7);
    config.temp_enabled = true;
    assert_eq!(config.fifo_message_len(), 8);
    config.stop_on_full = true;
    assert_eq!(config.fifo_message_len(), 8);
}

#[test]
fn test_interrupt_config_values() {
    let mut config = InterruptMapConfigBuilder::default().build();
    assert_eq!(config.map1(), 0u16);
    assert_eq!(config.map2(), 0u16);
    config = InterruptMapConfigBuilder::default()
        .fifo_watermark(bmi323::InterruptMapping::Int1)
        .build();
    assert_eq!(config.map1(), 0u16);
    assert_eq!(config.map2(), 1u16 << 12);
    config = InterruptMapConfigBuilder::default()
        .acc_drdy(bmi323::InterruptMapping::Int1)
        .gyr_drdy(bmi323::InterruptMapping::Int2)
        .build();
    assert_eq!(config.map1(), 0u16);
    assert_eq!(config.map2(), (2u16 << 8) | (1u16 << 10));
}

#[test]
fn test_interrupt_source(){
    assert_eq!(InterruptSource::from(1<<12), InterruptSource{gyr_drdy:true, ..Default::default()});
    assert_eq!(InterruptSource::from(1<<13), InterruptSource{acc_drdy:true, ..Default::default()});
    assert_eq!(InterruptSource::from(1<<14), InterruptSource{fifo_watermark:true, ..Default::default()});
    assert_eq!(InterruptSource::from(1<<15), InterruptSource{fifo_full:true, ..Default::default()});
    assert_eq!(InterruptSource::from(1<<12|1<<13), InterruptSource{gyr_drdy:true,acc_drdy:true, ..Default::default()});
}
