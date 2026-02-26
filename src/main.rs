mod audio;
mod utils;
use cpal::traits::{HostTrait, DeviceTrait};
use audio::player::{Player, PlayerCommand};

fn main() {
    let mut path = std::path::PathBuf::from("/home/lizuojun/Music/死亡搁浅/Bones(死亡搁浅).mp4");
    let player = Player::new();
    player.load(path);
    std::thread::sleep(std::time::Duration::from_secs(5));
    player.seek(210120);
    std::thread::sleep(std::time::Duration::from_secs(10));
}
