use cpal::{
    traits::{HostTrait, DeviceTrait, StreamTrait},
    Device,
    Stream,
    StreamConfig
};
use ringbuf::{
    producer::Producer,
    traits::Split, HeapRb};
use crossbeam_channel::{unbounded, Sender, Receiver};
use crate::{
    audio::decoder::AudioDecoder,
    app::event::{MainEvent, PlayerEvent }
};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}}
};


pub enum PlayerCommand {
    Load(PathBuf),  // Load a new audio file
    Play,           // playing
    Pause,          // Pause playback
    Seek(u64),      // Seek to a specific position
    Stop,           // Stop playback
    SetVolume(f32),  // Set volume
    SendFinishedEvent
}

pub struct PlaybackState {
    pub is_playing: AtomicBool,
    pub volume: AtomicU32,
    pub current_position: AtomicU64,
    pub total_duration_secs: AtomicU64
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            volume: AtomicU32::new(100), // 默认音量为 100%
            current_position: AtomicU64::new(0),
            total_duration_secs: AtomicU64::new(0)
        }
    }
}

pub struct Player {
    command_sender: Sender<PlayerCommand>,
    pub state: Arc<PlaybackState>,
    audio_thread: JoinHandle<()>
}

impl Player {
    pub fn new(event_sender: Sender<MainEvent>) -> Self {
        let (command_sender, command_receiver) = unbounded();
        let thread_command_sender = command_sender.clone();
        let state = Arc::new(PlaybackState::default());
        let audio_thread_state = state.clone();
        let audio_thread = std::thread::spawn(move || {
            Self::audio_loop(audio_thread_state, thread_command_sender, command_receiver, event_sender)
        });

        Self {
            command_sender,
            state,
            audio_thread
        }
    }

    pub fn load(&self, path: PathBuf) {
        self.command_sender.send(PlayerCommand::Load(path)).expect("Failed to send Load command");
    }

    pub fn play(&self) {
        self.command_sender.send(PlayerCommand::Play).expect("Failed to send Play command");
    }

    pub fn pause(&self) {
        self.command_sender.send(PlayerCommand::Pause).expect("Failed to send Pause command");
    }

    pub fn seek(&self, position_secs: u64) {
        self.command_sender.send(PlayerCommand::Seek(position_secs)).expect("Failed to send Seek command");
    }

    pub fn stop(&self) {
        self.command_sender.send(PlayerCommand::Stop).expect("Failed to send Stop command");
    }

    pub fn set_volume(&self, volume: f32) {
        self.command_sender.send(PlayerCommand::SetVolume(volume)).expect("Failed to send SetVolume command");
    }

    fn audio_loop(state: Arc<PlaybackState>, command_sender: Sender<PlayerCommand>, command_receiver: Receiver<PlayerCommand>, event_sender: Sender<MainEvent>) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let mut stream: Option<Stream> = None;
        let mut decode_thread: Option<JoinHandle<()>> = None;
        let mut current_path: Option<PathBuf> = None;

        loop {
            match command_receiver.recv() {
                Ok(PlayerCommand::Load(path)) => {
                    Self::process_load(command_sender.clone(), path, &mut current_path, &mut stream, &mut decode_thread, &device, state.clone());
                },
                Ok(PlayerCommand::SetVolume(volume)) => {
                    Self::process_set_volume(volume, state.clone());
                },
                Ok(PlayerCommand::Play) => {
                    Self::process_play(state.clone());
                },
                Ok(PlayerCommand::Pause) => {
                    Self::process_pause(state.clone());
                },
                Ok(PlayerCommand::Stop) => {
                    Self::process_stop(&mut stream, &mut decode_thread, state.clone());
                },
                Ok(PlayerCommand::Seek(position_secs)) => {
                    Self::process_seek(command_sender.clone(), current_path.clone().expect("seek before load"), position_secs, &mut stream, &mut decode_thread, &device, state.clone());
                },
                Ok(PlayerCommand::SendFinishedEvent) => {
                    Self::process_finished(&mut stream, &mut decode_thread, state.clone(), &event_sender);
                },
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    }

    fn process_finished(stream: &mut Option<Stream>, decode_thread: &mut Option<JoinHandle<()>>, state: Arc<PlaybackState>, event_sender: &Sender<MainEvent>) {
        event_sender.send(MainEvent::Player(PlayerEvent::SongFinished)).expect("Failed to send SongFinished event");
        *stream = None;
        *decode_thread = None;
        state.is_playing.store(false, Ordering::Relaxed);
        state.current_position.store(0, Ordering::Relaxed);
    }

    fn process_load(command_sender: Sender<PlayerCommand>, path: PathBuf, current_path: &mut Option<PathBuf>, stream: &mut Option<Stream>, decode_thread: &mut Option<JoinHandle<()>>, device: &Device, state: Arc<PlaybackState>) {
        *current_path = Some(path.clone());
        Self::load_and_play_at(command_sender, path, None, stream, decode_thread, device, state);
    }

    fn process_set_volume(volume: f32, state: Arc<PlaybackState>) {
        if !(0.0..=100.0).contains(&volume) {
            eprintln!("Volume must be between 0 and 100");
            return;
        }
        state.volume.store(volume as u32, Ordering::Relaxed);
    }

    fn process_play(state: Arc<PlaybackState>) {
        state.is_playing.store(true, Ordering::Relaxed);
    }

    fn process_pause(state: Arc<PlaybackState>) {
        state.is_playing.store(false, Ordering::Relaxed);
    }

    fn process_stop(stream: &mut Option<Stream>, decode_thread: &mut Option<JoinHandle<()>>, state: Arc<PlaybackState>) {
        *stream = None;
        *decode_thread = None;
        state.is_playing.store(false, Ordering::Relaxed);
        state.current_position.store(0, Ordering::Relaxed);
    }

    /// TODO: 错误处理，如果 seek 失败，应该保持当前播放状态不变，并输出错误信息
    fn process_seek(command_sender: Sender<PlayerCommand>, path: PathBuf, position_secs: u64, stream: &mut Option<Stream>, decode_thread: &mut Option<JoinHandle<()>>, device: &Device, state: Arc<PlaybackState>) {
        Self::load_and_play_at(command_sender, path, Some(position_secs), stream, decode_thread, device, state);
    }

    fn load_and_play_at(command_sender: Sender<PlayerCommand>, path: PathBuf, position_secs: Option<u64>, stream: &mut Option<Stream>, decode_thread: &mut Option<JoinHandle<()>>, device: &Device, state: Arc<PlaybackState>) {
        *stream = None;
        *decode_thread = None;

        let mut new_decoder = AudioDecoder::new(path);
        if let Some(pos) = position_secs && !new_decoder.seek(pos) {
            eprintln!("Failed to seek to position: {} seconds", pos);
        }
        let sample_rate = new_decoder.get_sample_rate();
        let channels = new_decoder.get_channels();

        let buffer_size = sample_rate as usize * channels as usize;
        let ring_buf = HeapRb::new(buffer_size);
        let (producer, consumer) = ring_buf.split();

        // decode packet

        let stream_config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default
        };
        let new_stream = Self::build_output_stream(device,
                                                           &stream_config, 
                                                           consumer, 
                                                           state.clone());
        // 是否需要在这里就调用 play 方法
        new_stream.play().expect("Failed to play stream");

        let new_decode_thread = Self::spawn_decode_thread(command_sender, new_decoder, producer);

        // 保证 stream 和 decode_thread 的生命周期足够长
        *stream = Some(new_stream);
        *decode_thread = Some(new_decode_thread);

        state.is_playing.store(true, Ordering::Relaxed);

    }

    fn build_output_stream<T>(device: &Device, stream_config: &StreamConfig, mut consumer: T, state: Arc<PlaybackState>) -> Stream 
        where
            T: ringbuf::traits::Consumer<Item = f32> + Send + 'static 
    {
        device.build_output_stream(stream_config, 
                                   move |data, _| {
                                       let is_playing = state.is_playing.load(Ordering::Relaxed);
                                       let volume_multiplier = state.volume.load(Ordering::Relaxed) as f32 / 100.0;
                                       for sample in data.iter_mut() {
                                           *sample = if is_playing {
                                               consumer.try_pop().unwrap_or(0.0) * volume_multiplier
                                           } else {
                                               0.0
                                           };
                                       }
                                   },
                                   |err| {
                                       panic!("Stream error: {:?}", err);
                                   }, 
                                   None)
                                   .expect("Failed to build output stream")
    }

    fn spawn_decode_thread<T>(command_sender: Sender<PlayerCommand>, mut decoder: AudioDecoder, mut producer: T)  -> JoinHandle<()>
        where 
            T: ringbuf::traits::Producer<Item = f32> + Send + 'static
    {
        thread::spawn(move || {
            while let Some(sample) = decoder.get_next_sample() {
                for s in sample {
                    while producer.try_push(s).is_err() {
                        thread::sleep(std::time::Duration::from_millis(900));
                    }
                }
            }
            command_sender.send(PlayerCommand::SendFinishedEvent).expect("Failed to send SendFinishedEvent command");
        })
    }
}

