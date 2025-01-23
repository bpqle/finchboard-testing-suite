mod lib;
use std::time::Duration;
use simple_logger::SimpleLogger;
use log::info;


#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let stepper = lib::StepperMotorApparatus::new("/dev/gpiochip1", "/dev/gpiochip3")
        .expect("StepperMotorApparatus Failed");
    info!("Apparatus created");
    stepper.switch_ctrl().await.unwrap();
    info!("Switch Control started");
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

}