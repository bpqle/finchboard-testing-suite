use gpio_cdev::{Chip, AsyncLineEventHandle,
                LineRequestFlags,
                LineHandle, MultiLineHandle,
                EventRequestFlags, EventType,
                errors::Error as GpioError};
use futures::stream::StreamExt;
use thiserror;
use log::info;

struct PeckLEDs {
    right_leds: MultiLineHandle,
    center_leds: MultiLineHandle,
    left_leds: MultiLineHandle,
    peck_position: Vec<LedState>,
}
struct PeckKeys {
    interrupt_line: u32
}
pub struct PeckBoard {
    leds: PeckLEDs,
    keys: PeckKeys,
}

impl PeckBoard {
    const INTERRUPT_CHIP: &'static str = "/dev/gpiochip2";
    const PECK_KEY_LINES: [u32; 3] = [13,14,15];

    pub async fn new (chip: &str) -> Result<Self, Error> {
        let mut chip = Chip::new(chip).map_err(|e:GpioError|
            Error::ChipError {source: e,
                chip: ChipNumber::Chip4}
        )?;

        let mut chip2 = Chip::new(&Self::INTERRUPT_CHIP)
            .map_err(|e:GpioError| Error::ChipError {source: e, chip: ChipNumber::Chip2})
            .unwrap();

        let line22 = chip2.get_line(22)
            .map_err(|e:GpioError| Error::LineGetError {source:e, line: 22}).unwrap();
        let mut events22 = AsyncLineEventHandle::new(line22.events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES, //Setting flags to FALLING_EDGE does not exclude RISING events??
            "async peckboard interrupt",
        ).unwrap()).unwrap();
        let line23 = chip2.get_line(23)
            .map_err(|e:GpioError| Error::LineGetError {source:e, line: 23}).unwrap();
        let mut events23 = AsyncLineEventHandle::new(line23.events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES, //Setting flags to FALLING_EDGE does not exclude RISING events??
            "async peckboard interrupt",
        ).unwrap()).unwrap();
        let line24 = chip2.get_line(24)
            .map_err(|e:GpioError| Error::LineGetError {source:e, line: 24}).unwrap();
        let mut events24 = AsyncLineEventHandle::new(line24.events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES, //Setting flags to FALLING_EDGE does not exclude RISING events??
            "async peckboard interrupt",
        ).unwrap()).unwrap();
        let line25 = chip2.get_line(25)
            .map_err(|e:GpioError| Error::LineGetError {source:e, line: 25}).unwrap();
        let mut events25 = AsyncLineEventHandle::new(line25.events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES, //Setting flags to FALLING_EDGE does not exclude RISING events??
            "async peckboard interrupt",
        ).unwrap()).unwrap();

        let interrupt_line: u32;
        info!("Initiating peckboard! Try pecking one of the keys.");
        tokio::select! {
            _ = events22.next() => {info!("Interrupted on line 22"); interrupt_line=22}
            _ = events23.next() => {info!("Interrupted on line 23"); interrupt_line=23}
            _ = events24.next() => {info!("Interrupted on line 24"); interrupt_line=24}
            _ = events25.next() => {info!("Interrupted on line 25"); interrupt_line=25}
        }

        let keys = PeckKeys::new(&mut chip, interrupt_line)?;
        let leds = PeckLEDs::new(&mut chip)?;

        Ok(PeckBoard{
            leds,
            keys
        })
    }
    pub async fn monitor(mut self) -> Result<(), Error> {
        tokio::spawn( async move {
            let mut chip2 = Chip::new(&Self::INTERRUPT_CHIP)
                .map_err(|e:GpioError| Error::ChipError {source: e, chip: ChipNumber::Chip2})
                .unwrap();
            let interrupt_line = chip2.get_line(self.keys.interrupt_line)
                .map_err(|e:GpioError| Error::LineGetError {source:e, line: self.keys.interrupt_line}).unwrap();
            let mut events = AsyncLineEventHandle::new(interrupt_line.events(
                LineRequestFlags::INPUT,
                EventRequestFlags::BOTH_EDGES, //Setting flags to FALLING_EDGE does not exclude RISING events??
                "async peckboard interrupt",
            ).unwrap()).unwrap();

            let mut chip4 = Chip::new("/dev/gpiochip4")
                .map_err(|e:GpioError| Error::ChipError {source: e, chip: ChipNumber::Chip4})
                .unwrap();
            let key_handles: MultiLineHandle = chip4.get_lines(&Self::PECK_KEY_LINES).unwrap()
                .request(LineRequestFlags::INPUT, &[0,0,0], "peck_keys").unwrap();

            loop {
                match events.next().await {
                    Some(event) => {
                        //println!("{:?}", event.unwrap().event_type())
                        match event.unwrap().event_type() {
                            EventType::RisingEdge => {continue},
                            EventType::FallingEdge => {
                                let values = key_handles.get_values().unwrap();
                                let position = values.iter().position(|&x| x == 1).unwrap_or(3);
                                self.leds.pecked(position).unwrap();
                            },
                        }
                    },
                    None => break,
                };
            }
        });
        Ok(())
    }

}
impl PeckLEDs {
    const RIGHT_LINES: [u32;3] = [0,3,6];
    const CENTER_LINES: [u32;3] = [1,4,7];
    const LEFT_LINES: [u32;3] = [2,5,8];

    pub fn new(chip: &mut Chip) -> Result<Self, Error> {
        let right_leds = chip.get_lines(&Self::RIGHT_LINES)
            .map_err(|e:GpioError| Error::LinesGetError {source: e, lines: &Self::RIGHT_LINES}).unwrap()
            .request(LineRequestFlags::OUTPUT, &LedState::Off.as_value(), "peck_leds")
            .map_err(|e:GpioError| Error::LinesReqError {source: e, lines: &Self::RIGHT_LINES}).unwrap();
        let center_leds = chip.get_lines(&Self::CENTER_LINES)
            .map_err(|e:GpioError| Error::LinesGetError {source: e, lines: &Self::CENTER_LINES}).unwrap()
            .request(LineRequestFlags::OUTPUT, &LedState::Off.as_value(), "peck_leds")
            .map_err(|e:GpioError| Error::LinesReqError {source: e, lines: &Self::CENTER_LINES}).unwrap();
        let left_leds = chip.get_lines(&Self::LEFT_LINES)
            .map_err(|e:GpioError| Error::LinesGetError {source: e, lines: &Self::LEFT_LINES}).unwrap()
            .request(LineRequestFlags::OUTPUT, &LedState::Off.as_value(), "peck_leds")
            .map_err(|e:GpioError| Error::LinesReqError {source: e, lines: &Self::LEFT_LINES}).unwrap();
        let peck_states: Vec<LedState> = vec![LedState::Off,LedState::Off,LedState::Off];

        Ok(PeckLEDs{
            right_leds,
            center_leds,
            left_leds,
            peck_position: peck_states
        })
    }
    pub fn pecked(&mut self, position: usize) -> Result<(), Error> {
        match position {
            0 => {
                self.peck_position[0].next();
                let led_state = &self.peck_position[0].as_value();
                self.right_leds.set_values(led_state)
                    .map_err(|e: GpioError| Error::LinesSetError {source:e, lines: &Self::RIGHT_LINES})
                    .unwrap()
            },
            1 => {
                self.peck_position[1].next();
                let led_state = &self.peck_position[1].as_value();
                self.center_leds.set_values(led_state)
                    .map_err(|e: GpioError| Error::LinesSetError { source: e, lines: &Self::CENTER_LINES })
                    .unwrap()
            },
            2 => {
                self.peck_position[2].next();
                let led_state = &self.peck_position[2].as_value();
                self.left_leds.set_values(led_state)
                    .map_err(|e: GpioError| Error::LinesSetError {source:e, lines: &Self::LEFT_LINES})
                    .unwrap()
            },
            _ => {}//println!("Invalid peck information")}
        }
        Ok(())
    }
}
impl PeckKeys {
    const IR: [u32; 3] = [9,10,11];
    pub fn new(chip: &mut Chip, interrupt_line: u32) -> Result<Self, Error> {
        let _ir_handles: Vec<LineHandle> = Self::IR.iter()
            .map(|&offset| {
                chip.get_line(offset).unwrap()
                    .request(LineRequestFlags::OUTPUT, 1, "peckboard_ir")
                    .unwrap()
            }).collect();
        Ok(PeckKeys{
            interrupt_line
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LedState {
    Off,
    Blue,
    Red,
    Green,
    All,
}
impl LedState {
    fn next(&mut self) -> &mut Self {
        match self {
            LedState::Off   => {*self = LedState::Blue}
            LedState::Blue   => {*self = LedState::Red}
            LedState::Red  => {*self = LedState::Green}
            LedState::Green => {*self = LedState::All}
            LedState::All   => {*self = LedState::Off}
        };
        self
    }
    fn as_value(&self) -> [u8; 3] {
        match self {
            LedState::Off => {[0,0,0]}
            LedState::Red => {[1,0,0]}
            LedState::Blue => {[0,1,0]}
            LedState::Green => {[0,0,1]}
            LedState::All => {[1,1,1]}
        }
    }
}
#[derive(Debug)]
pub enum ChipNumber {
    Chip2,
    Chip4,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get chip {chip:?}")]
    ChipError {
        source: GpioError,
        chip: ChipNumber,
    },
    #[error("Failed to get line")]
    LineGetError {
        source: GpioError,
        line: u32,
    },
    #[error("Failed to get lines")]
    LinesGetError {
        source: GpioError,
        lines: &'static [u32],
    },
    #[error("Failed to request lines")]
    LinesReqError {
        source: GpioError,
        lines: &'static [u32],
    },
    #[error("Failed to set lines")]
    LinesSetError {
        source: GpioError,
        lines: &'static [u32],
    },
}