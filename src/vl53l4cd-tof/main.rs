use vl53l4cd::Vl53l4cd;
use i2cdev::linux::LinuxI2CBus;
use linux_embedded_hal_async::{delay, i2c};
use argh::{self, FromArgs};
use simple_logger::SimpleLogger;
use log::info;

#[derive(FromArgs)]
/// Get distance readings from the ToF sensor
struct CliArgs {
    /// the timing budget between 10 and 200ms, defaults to 50ms
    #[argh(option, default = "default_budget()")]
    budget: u32,
    /// interval between readings, must be lower than timing budget, defaults to 0
    #[argh(option, default = "default_interval()")]
    interval: u32,
}
fn default_budget() -> u32 {50}
fn default_interval() -> u32 {0}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
    let args: CliArgs = argh::from_env();
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
    sensor.set_range_timing(args.budget, args.interval).await.unwrap();
    sensor.start_ranging().await.unwrap();
    loop {
        let measure = sensor.measure().await.unwrap();
        if !measure.is_valid() {
            info!("Measurement not valid {:?}", measure)
        } else {
            info!("Distance is {}", measure.distance)
        }
    }
}