mod lib;
use std::fs;
use std::time::Duration;
use std::path::{Path, PathBuf};
use simple_logger::SimpleLogger;
use log::info;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    if !Path::new("/sys/class/i2c-adapter/i2c-1/1-0020").exists() {
        info!("Manually exporting device to i2c.");
        let chip_path = String::from("/sys/class/i2c-adapter/i2c-1/new_device");
        let sysfs_chip = fs::canonicalize(PathBuf::from(chip_path.clone()))
            .unwrap();
        fs::write(sysfs_chip.clone(), "pcf8575 0x20")
            .unwrap();
        assert!(Path::new("/sys/class/i2c-adapter/i2c-1/1-0020").exists());
    }
    // Give it a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    let peck_board = lib::PeckBoard::new("/dev/gpiochip4").await
        .expect("Couldn't initialize peckboard chip with gpio. Check if device is plugged in correctly");
    peck_board.monitor().await.unwrap();
    info!("PeckBoard initiated. Cycle through leds by pecking.");
    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }

}