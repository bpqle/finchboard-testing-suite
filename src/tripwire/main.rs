use vl53l4cd::Vl53l4cd;
use linux_embedded_hal_async as hal;
use i2cdev::linux::LinuxI2CBus;
use hal::{delay, i2c};

#[tokio::main]
async fn main() {
    let dev = i2c::LinuxI2c::new(
        LinuxI2CBus::new("/dev/i2c-2").unwrap()
    );
    let mut sensor = Vl53l4cd::new(
        dev,
        delay::LinuxDelay,
        vl53l4cd::wait::Poll,
    );
    // sensor.set_range_timing(30, 30).await.unwrap();
    sensor.init().await.unwrap();
    sensor.start_ranging().await.unwrap();
    loop {
        let measure = sensor.measure().await.unwrap();
        if !measure.is_valid() {
            println!("Measurement not valid {:?}", measure)
        } else {
            println!("Distance is {}", measure.distance)
        }
    }
}