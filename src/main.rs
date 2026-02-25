mod audio;
mod utils;
use cpal::traits::{HostTrait, DeviceTrait};

fn main() {
    let path = std::path::PathBuf::from("/home/lizuojun/Music/死亡搁浅/Don't Be So Serious(死亡搁浅).mp4");
    let player = audio::player::Player::new(path);
    std::thread::sleep(std::time::Duration::from_secs(300))
}
