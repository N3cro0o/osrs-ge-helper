/// Module used to handle, play and store audio files. All audio files are embeded into executable
/// file using include_bytes! macro.

use crate::{log_mess, log_err};

use std::sync::OnceLock;
use std::io::{Cursor, BufReader};
use std::collections::HashMap;
use rodio::{Decoder, DeviceSinkBuilder, Player};
use rodio::stream::MixerDeviceSink;

/// Main application audio mixer is stored in AUDIO_SINK static variable. Used to play audio notifications.
static AUDIO_SINK: OnceLock<Option<MixerDeviceSink>> = OnceLock::new();
static AUDIO_PLAYER: OnceLock<Player> = OnceLock::new();
static AUDIO_MAP: OnceLock<HashMap<String, Vec<u8>>> = OnceLock::new();

pub fn audio_init() -> Result<(), String> {
    match DeviceSinkBuilder::open_default_sink() {
        Ok(mixer) => {
            let _ = AUDIO_PLAYER.set(Player::connect_new(mixer.mixer()));
            let _ = AUDIO_SINK.set(Some(mixer));
            log_mess!["Audio sink ready"];
        }
        Err(err) => {
            let _ = AUDIO_SINK.set(None);
            let str = format!("Audio sink error: {}", err);
            return Err(str);
        }
    }; 
    prepare_audio_map()?;
    Ok(()) 
}

/// Auxiliary function used to find audio files, include them inside executable file and store them
/// inside AUDIO_MAP HashMap static variable
fn prepare_audio_map() -> Result<(), String> {
    // let ping_audio_vec = BufReader::new(Cursor::new(include_bytes!("../sfx/wet-bell-shot.wav").to_vec()));
    // let ping_audio_decoder = match Decoder::builder().with_data(ping_audio_vec).with_hint("wav").build() {
    //     Ok(d) => d,
    //     Err(err) => { return Err(err.to_string()); }
    // };
    let mut map = HashMap::new();
    map.insert("ping".to_string(), include_bytes!("../sfx/wet-bell-shot.mp3").to_vec());
    map.insert("tory_przyszlosc".to_string(), include_bytes!("../sfx/tory_to_jest_przyszlosc.mp3").to_vec());
    if let Err(err) = AUDIO_MAP.set(map) {
        return Err(format!("{:?}", err));
    }
    log_mess!["Audio tracks ready"];
    Ok(())
}

/// Main function used to play given audio track using its name. Additionaly volume argument will
/// change player playback volume on each function call.
///
/// Audio won't be played or queued.
pub fn play_audio(name: &str, volume: f32) -> Result<(), String> {
    log_mess!["Play audio: {}", name];
    let player = match AUDIO_PLAYER.get() {
        Some(p) => p,
        None => { return Err("No player found".to_string()); }
    };
    if !player.empty() {
        return Err("Audio player is not empty".to_string());
    }
    let audio_map = match AUDIO_MAP.get() {
        Some(m) => m,
        None => { return Err("No HashMap found".to_string()); }
    };
    let audio = match audio_map.get(name) {
        Some(v) => v,
        None => { return Err(format!("No audio file with name {}", name));}
    };
    let decoder = match Decoder::builder().with_data(BufReader::new(Cursor::new(audio))).build() {
        Ok(d) => d,
        Err(err) => { return Err(err.to_string()); }
    };
    player.set_volume(volume);
    player.append(decoder);
    Ok(())
}

/// Helper function used to streamline calling `play_audio()` function to allow the User to quickly
/// check the set parameters
pub fn test_audio(volume: f32) -> Result<(), String> {
    let numbah = rand::random_range(0 ..= 2137);
    let name;
    if numbah == 999 {
        name = "tory_przyszlosc";
    }
    else {
        name = "ping";
    }
    play_audio(name, volume)?;
    Ok(())
}

/// Debug `test_audio()` function call without error handling
pub fn check_audio_debug() {
    let player = AUDIO_PLAYER.get().unwrap();
    let audio = AUDIO_MAP.get().unwrap().get("ping").unwrap();
    let decoder = match Decoder::builder().with_data(BufReader::new(Cursor::new(audio))).build() {
        Ok(d) => d,
        Err(err) => { log_err!["Error while decoding 'ping' audio bytes: {}", err]; return; }
    };
    player.append(decoder);
}

#[allow(dead_code)]
fn linear_to_db(linear: f32) -> f32 {
    linear.log10() * 10.0
}
