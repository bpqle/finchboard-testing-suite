use std::fs;
use std::io;
use std::path::{self, PathBuf, Path};
use std::ptr::write;
use std::time::{self, SystemTime};
use chrono::{self, Timelike, prelude::*};
use argh::{self,FromArgs};

#[derive(FromArgs)]
///
struct CliArgs {
    /// whether to control the main LED through the pwm interface or sysfs device
    #[argh(switch, short='p')]
    pwm: bool,
    /// fake dawn value, set to 8AM
    #[argh(option, default = 8.0)]
    dawn: f64,
    /// fake dusk value, set to 8PM
    #[argh(option, default = 20.0)]
    dusk: f64,
}

fn main() {
    let args: CliArgs = argh::from_env();
    let mut write_mode = args.pwm; // False if writing to sysfs, true if writing to pwm
    let mut write_loc: String;

    if !write_mode & Path::new(String::from("/sys/class/leds/starboard::lights/brightness")).exists() {
        println!("Sysfs device not found at {:?}. ", sysfs_dev_path);
        pwm_setup();
        write_mode = true;
        write_loc = String::from("/sys/class/pwm/pwmchip2/pwm1/duty_cycle");
    } else if write_mode {
        pwm_setup();
        write_loc = String::from("/sys/class/pwm/pwmchip2/pwm1/duty_cycle");
    } else {
        write_loc = String::from("/sys/class/leds/starboard::lights/brightness");
    }
    let path = fs::canonicalize(PathBuf::from(write_loc)).unwrap();

    loop {
        let mut buffer = String::new();
        println!("Input brightness level between 0 and 255, or 'auto' for artificial light cycle value");
        io::stdin().read_line(&mut buffer)?;
        if buffer == String::from("auto") {
            let altitude = calc_altitude(args.dawn, args.dusk);
            let brightness = calc_brightness(altitude, 255);
            if write_mode {
                let duty_cycle = 500000 * (1 - brightness as u32 / 255);
                println!("Writing {:?} to file {:?}", duty_cycle, write_loc);
                fs::write(path, duty_cycle.to_string()).expect("Unable to write value to file");
            } else {
                println!("Writing {:?} to file {:?}", brightness, write_loc);
                fs::write(path, brightness.to_string()).expect("Unable to write value to file");
            }
        } else {
            let brightness: u8 = buffer.trim().parse().expect("Input not an integer.");
            if write_mode {
                let duty_cycle = 500000 * (1 - brightness as u32 / 255);
                println!("Writing {:?} to file {:?}", duty_cycle, write_loc);
                fs::write(path, duty_cycle.to_string()).expect("Unable to write value to file");
            } else {
                println!("Writing {:?} to file {:?}", brightness, write_loc);
                fs::write(path, brightness.to_string()).expect("Unable to write value to file");
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
        fs::write(export_loc.clone(), "1").map_err(|_e| DecideError::Component {
            source:
            HouseLightError::WriteError { path: export_loc.clone(), value: "1".to_string() }.into()
        }).unwrap()
    }
    let configs = vec!["period", "500000", "polarity", "inversed", "enable", "1"];
    for pair in configs.chunks(2) {
        let write_loc = format!("{}{}", pwm_address, pair[0]);
        fs::write(write_loc.clone(), pair[1])
            .map_err(|_e| DecideError::Component {
                source:
                HouseLightError::WriteError { path: write_loc, value: pair[1].to_string() }.into()
            }).unwrap()
    }
}


fn calc_altitude(dawn:f64, dusk:f64) -> f64 {
    let now: DateTime<Local> = Local::now();
    let hour = now.hour() as f64;
    let minute = now.minute() as f64;
    let second = now.second() as f64;
    let time = hour + minute / 60.0 + second / 3600.0;
    println!("Time is {}", time);
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

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    components: Vec<Component>
}

#[derive(Debug, Serialize, Deserialize)]
struct Component {
    dawn: Option<f64>,
    dusk: Option<f64>
}