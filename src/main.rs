mod audio;
mod utils;
use cpal::traits::{HostTrait, DeviceTrait};

fn main() {
    let path = std::path::PathBuf::from("/home/lizuojun/Music/Darlin' - BENI.mp3");
    let player = audio::player::Player::new(path);
    std::thread::sleep(std::time::Duration::from_secs(300))
}
