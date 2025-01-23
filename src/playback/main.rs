use alsa::Direction;
use alsa::pcm::{PCM, HwParams, Format, Access, State};
use std::path::Path;
use argh::{self,FromArgs};
use sndfile::{ReadOptions, SndFileIO};
use simple_logger::SimpleLogger;
use log::info;
use sndfile;

#[derive(FromArgs)]
/// Playback an audio file with the default audio device
#[argh(help_triggers("-h", "--help", "help"))]
struct CliArgs {
    #[argh(positional)]
    /// path to wav file
    wav_file: String,
    #[argh(option, default = "default_device()", short='d')]
    /// playback device, defaults to plughw:1
    device: String,
    #[argh(option, default = "default_card()", short='c')]
    /// playback card, used for volume changing, defaults to hw:1
    card: String,
    #[argh(option, default="default_rate()", short='s')]
    /// sampling rate, defaults to 44100
    sample_rate: u32,
    #[argh(option, default = "default_channel()", short='m')]
    /// channel count, 1 for mono and 2 for stereo
    channel: usize,
    #[argh(option, default = "default_volume()", short='v')]
    /// volume setting between 0 and 100
    volume: i64,
}

fn default_device() -> String {String::from("plughw:1")}
fn default_card() -> String {String::from("hw:1")}
fn default_channel() -> usize {1}
fn default_volume() -> i64 {100}
fn default_rate() -> u32 {44100}

fn main() {

    SimpleLogger::new().init().unwrap();
    let args: CliArgs = argh::from_env();
    let audio_path = Path::new(&args.wav_file).canonicalize().unwrap();
    let fname = audio_path.file_stem().unwrap().to_os_string();
    info!("loading file {:?}", fname);

    let stimulus: Vec<i16>;
    if audio_path.extension().is_some_and(|ext| ext=="wav") {
        let mut audio_file = sndfile::OpenOptions::ReadOnly(ReadOptions::Auto)
            .from_path(audio_path).unwrap();
        let wav: Vec<i16> = audio_file.read_all_to_vec().unwrap();
        let wav_channels = audio_file.get_channels();
        stimulus = process_audio(wav, wav_channels, args.channel);
    } else {info!("Provided file not an audio file!"); return}

    let audio_dev = PCM::new(&args.device.clone(), Direction::Playback, false).unwrap();
    info!("pcm device created.");
    get_hw_config(&audio_dev, args.channel, args.sample_rate).unwrap();

    let mixer = alsa::mixer::Mixer::new(&args.card.clone(), false).unwrap();
    let selem_id = alsa::mixer::SelemId::new("PCM", 0);
    let selem = mixer.find_selem(&selem_id).ok_or_else(|| {
        format!(
            "Couldn't find selem with name '{}'.",
            selem_id.get_name().unwrap_or("unnamed")
        )
    }).unwrap();
    selem.set_playback_volume_range(0, 100).unwrap();
    selem.set_playback_volume_all(args.volume).unwrap();
    drop(mixer);

    let mut io = audio_dev.io_i16().unwrap();
    match audio_dev.prepare() {
        Ok(n) => n,
        Err(e) => {
            info!("failed to prepare playback device. recovering.");
            audio_dev.recover(e.errno() as std::os::raw::c_int, true).unwrap()
        }
    }
    info!("staring playback");
    let _ = playback_io(&audio_dev, &mut io, &stimulus);
    info!("complete!")
}


fn process_audio(wav: Vec<i16>, wav_channels: usize, hw_channels: usize) -> Vec<i16> {
    let mut result = Vec::new();
    if wav_channels == 1 {
        result = wav;
        if hw_channels == 2 {
            result = result.into_iter()
                .map(|note| [note, note])
                .flatten()
                .map(|f| f )
                .collect::<Vec<i16>>()
        }
    } else if wav_channels == 2 {
        result = wav;
        if hw_channels == 1 {
            result = result.into_iter()
                .enumerate()
                .filter(|f| f.0.clone() % 2 == 0)
                .map(|f| f.1)
                .collect::<Vec<_>>()
        }
    };
    result
}


fn get_hw_config<'a>(pcm: &'a PCM, channel: usize, sampling_rate: u32) -> Result<bool, String>{
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(channel as u32).unwrap();
    hwp.set_rate(sampling_rate, alsa::ValueOr::Nearest).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_buffer_size(1024).unwrap();
    // hwp.set_period_size(512, alsa::ValueOr::Nearest).unwrap();
    pcm.hw_params(&hwp).unwrap();
    Ok(true)
}

fn playback_io(pcm: &PCM, io: &mut alsa::pcm::IO<i16>, data: &Vec<i16>)
                   -> Result<bool, String> {
    let frames: usize = data.len();
    let _avail = match pcm.avail_update() {
        Ok(n) => n,
        Err(e) => {
            info!("sound-alsa failed to call available update, recovering from {}", e);
            pcm.recover(e.errno() as std::os::raw::c_int, true).unwrap();
            pcm.avail_update().unwrap()
        }
    } as usize;
    let mut pointer = 0;
    let mut _written: usize = 0;
    //loop while playing
    while pointer < frames-1 {
        let slice = if pointer+512>frames {&data[pointer..]} else {&data[pointer..pointer+512]};
        _written = match io.writei(slice) {
            Ok(n) => n,
            Err(e) => {
                info!("Recovering from {}", e);
                pcm.recover(e.errno() as std::os::raw::c_int, true).unwrap();
                0
            }
        };
        pointer += _written;
        match pcm.state() {
            State::Running => {
            }, // All fine
            State::Prepared => {
                pcm.start().unwrap();
            },
            State::XRun => {
                info!("underrun in audio output stream!, will call prepare()");
                pcm.prepare().unwrap();
            },
            State::Suspended => {
                info!("sound-alsa suspended, will call prepare()");
                pcm.prepare().unwrap();
            },
            n @ _ => panic!("sound-alsa unexpected pcm state {:?}", n),
        };
    };
    Ok(true)
}