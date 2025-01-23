use std::fs;
use std::io;
use std::path::{PathBuf, Path};
use chrono::{self, Timelike, prelude::*};
use argh::{self,FromArgs};
use simple_logger::SimpleLogger;
use log::info;

#[derive(FromArgs)]
/// Manually control house light LED
struct CliArgs {
    /// fake dawn value, defaults to 8AM
    #[argh(option, default = "default_dawn()")]
    dawn: f64,
    /// fake dusk value, defaults to 8PM
    #[argh(option, default = "default_dusk()")]
    dusk: f64,
}

fn default_dawn() -> f64 {8.0}
fn default_dusk() -> f64 {8.0}

fn main() {
    SimpleLogger::new().init().unwrap();

    let args: CliArgs = argh::from_env();
    let write_loc: String;
    let write_mode: bool;
    if !Path::new("/sys/class/leds/starboard::lights/brightness").exists() {
        info!("Sysfs device not found for house light. Defaulting to pwm method.");
        pwm_setup();
        write_mode = true;
        write_loc = String::from("/sys/class/pwm/pwmchip2/pwm1/duty_cycle");
    } else {
        write_mode = false;
        info!("Changing house light LED with sysfs device tree.");
        write_loc = String::from("/sys/class/leds/starboard::lights/brightness");
    }
    let path = fs::canonicalize(PathBuf::from(write_loc.clone())).unwrap();

    loop {
        let mut buffer = String::new();
        info!("Input brightness level between 0 and 255, or 'auto' for artificial light cycle value");
        io::stdin().read_line(&mut buffer).unwrap();
        if buffer.trim() == String::from("auto") {
            let altitude = calc_altitude(args.dawn, args.dusk);
            let brightness = calc_brightness(altitude, 255);
            if write_mode {
                let duty_cycle = 500000 * (1 - brightness as u32 / 255);
                info!("Writing {:?} to file {:?}", duty_cycle, write_loc);
                fs::write(path.clone(), duty_cycle.to_string()).expect("Unable to write value to file");
            } else {
                info!("Writing {:?} to file {:?}", brightness, write_loc);
                fs::write(path.clone(), brightness.to_string()).expect("Unable to write value to file");
            }
        } else {
            let brightness: u8 = buffer.trim().parse().expect("Input not an integer.");
            if write_mode {
                let duty_cycle = 500000 * (1 - brightness as u32 / 255);
                info!("Writing {:?} to file {:?}", duty_cycle, write_loc);
                fs::write(path.clone(), duty_cycle.to_string()).expect("Unable to write value to file");
            } else {
                info!("Writing {:?} to file {:?}", brightness, write_loc);
                fs::write(path.clone(), brightness.to_string()).expect("Unable to write value to file");
            }
        }
    }
}

fn pwm_setup() {
    // Set up the pwm device
    // Brightness can be adjusted by writing to the duty_cycle to be a proportion of the period
    let pwm_address: String = String::from("/sys/class/pwm/pwmchip2/pwm1/");
    if !Path::new(&pwm_address).exists() {
        let export_loc = String::from("/sys/class/pwm/pwmchip2/export");
        fs::write(export_loc.clone(), "1").unwrap()
    }
    let configs = vec!["period", "500000", "polarity", "inversed", "enable", "1"];
    for pair in configs.chunks(2) {
        let write_loc = format!("{}{}", pwm_address, pair[0]);
        fs::write(write_loc.clone(), pair[1]).unwrap()
    }
}


fn calc_altitude(dawn:f64, dusk:f64) -> f64 {
    let now: DateTime<Local> = Local::now();
    let hour = now.hour() as f64;
    let minute = now.minute() as f64;
    let second = now.second() as f64;
    let time = hour + minute / 60.0 + second / 3600.0;
    info!("Time is {}", time);
    let x: f64 = (time + 24.0 - dawn) % 24.0;
    let y: f64 = (dusk + 24.0 - dawn) % 24.0;
    let alt = (x / y) * std::f64::consts::PI;
    alt
}


fn calc_brightness(altitude: f64, max_brightness: u8) -> i8 {
    let x = altitude.sin() * (max_brightness as f64);
    let y = x.round() as i8;
    if y > 0 { y } else { 0 }
}