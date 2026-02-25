mod audio;
mod utils;

fn main() {
    let file_path_list = utils::file_manager::FileManager::get_file_path_list_static(std::path::PathBuf::from("/home/lizuojun/Music/"));
    println!("{:?}", file_path_list.first().unwrap().display());
    let mut audio_decoder = audio::decoder::AudioDecoder::new(file_path_list.first().unwrap().clone());
    let packet = audio_decoder.get_next_sample();
    println!("{}", packet.unwrap().len());
}
