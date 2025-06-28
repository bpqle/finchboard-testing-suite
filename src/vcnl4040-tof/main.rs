// Example usage (commented out since it requires specific hardware)
use vcnl4040::{LedCurrent, LedDutyCycle, ProximityIntegrationTime, Vcnl4040};
use linux_embedded_hal_async::{i2c};
use i2cdev::linux::LinuxI2CBus;
// use argh::{self, FromArgs};
use simple_logger::SimpleLogger;
use log::info;
use std::time::Duration;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
    let dev = i2c::LinuxI2c::new(
        LinuxI2CBus::new("/dev/i2c-2").unwrap()
    );

    let mut sensor = Vcnl4040::new(dev);
    sensor.init(false).await.unwrap();
    sensor.enable_proximity(true).await.unwrap();
    sensor.set_proximity_led_current(LedCurrent::Current100mA).await.unwrap();
    sensor.set_proximity_led_duty_cycle(LedDutyCycle::Duty1_160).await.unwrap();
    sensor.set_proximity_integration_time(ProximityIntegrationTime::Time2T).await.unwrap();

    loop {
        let dist = sensor.get_proximity().await.unwrap();
        info!("distance is {}", dist);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
