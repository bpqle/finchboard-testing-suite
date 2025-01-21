use peckboard_test::{PeckBoard};
use std::thread;
use std::time::Duration;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {

    if !Path::new("/sys/class/i2c-adapter/i2c-1/1-0020").exists() {
        let chip_path = String::from("/sys/class/i2c-adapter/i2c-1/new_device");
        let sysfs_chip = fs::canonicalize(PathBuf::from(chip_path.clone()))
            .unwrap();
        fs::write(sysfs_chip.clone(), "pcf8575 0x20")
            .unwrap();
        tracing::debug!("peckboard gpio chip initiated");
        assert!(Path::new("/sys/class/i2c-adapter/i2c-1/1-0020").exists());
    }

    let peck_board = PeckBoard::new("/dev/gpiochip4").await
        .expect("Couldn't initialize PeckBoard chip");
    peck_board.monitor().await.unwrap();
    loop {
        println!("PeckBoard initiated. Test by pecking manually");
        thread::sleep(Duration::from_secs(100));
    }

}