use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use ringbuf::{traits::Split, HeapRb};
use crossbeam_channel::{unbounded, Sender, Receiver};
use crate::audio::decoder::AudioDecoder;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

pub enum PlayerCommand {
    Load(PathBuf),  // Load a new audio file
    Play,           // playing
    Pause,          // Pause playback
    Seek(u64),      // Seek to a specific position
    Stop,           // Stop playback
    SetVolume(f32)  // Set volume
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
    state: Arc<PlaybackState>,
    audio_thread: JoinHandle<()>
}

impl Player {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = unbounded();
        let state = Arc::new(PlaybackState::default());
        let audio_thread_state = state.clone();
        let audio_thread = std::thread::spawn(move || {
            Self::audio_loop(audio_thread_state, command_receiver)
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

    }

    pub fn pause(&self) {

    }

    pub fn seek(&self, position_secs: u64) {

    }

    pub fn stop(&self) {

    }

    pub fn set_volume(&self, volume: f32) {
    }

    fn audio_loop(state: Arc<PlaybackState>, command_receiver: Receiver<PlayerCommand>) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let mut stream: Option<Stream> = None;
        let mut decode_thread: Option<JoinHandle<()>> = None;

        loop {
            match command_receiver.try_recv() {
                Ok(PlayerCommand::Load(path)) => {
                    println!("Loading file: {:?}", path);
                    Self::load_process(path, &mut stream, &mut decode_thread, &device, state.clone());
                },
                Ok(_) => {
                    panic!("not implemented");
                },
                Err(_) => {
                    print!("-");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    }

    fn load_process(path: PathBuf, stream: &mut Option<Stream>, decode_thread: &mut Option<JoinHandle<()>>, device: &Device, state: Arc<PlaybackState>) {
        *stream = None;
        *decode_thread = None;

        let new_decoder = AudioDecoder::new(path);
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

        let new_decode_thread = Self::spawn_decode_thread(new_decoder, producer);

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

    fn spawn_decode_thread<T>(mut decoder: AudioDecoder, mut producer: T)  -> JoinHandle<()>
        where
            T: ringbuf::traits::Producer<Item = f32> + Send + 'static
    {
        thread::spawn(move || {
            while let Some(sample) = decoder.get_next_sample() {
                for s in sample {
                    while producer.try_push(s).is_err() {
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        })
    }
}

/*

pub struct Player {
    _stream: Stream,
    _decode_thread: JoinHandle<()>
}

impl Player {
    pub fn new(file_path: PathBuf) -> Self {
        let decoder = AudioDecoder::new(file_path);
        let sample_rate = decoder.get_sample_rate();
        let channels = decoder.get_channels();

        let buffer_size = sample_rate as usize * channels as usize;
        let ring_buf = HeapRb::new(buffer_size);
        let (producer, consumer) = ring_buf.split();

        let stream_config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default
        };

        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let stream = Self::build_output_stream(&device, &stream_config, consumer);

        let decode_thread = Self::spawn_decode_thread(decoder, producer);
        stream.play().expect("Failed to play stream");

        Self {
            _stream: stream,
            _decode_thread: decode_thread
        }
    }

    fn build_output_stream<T>(device: &Device, stream_config: &StreamConfig, mut consumer: T) -> Stream 
        where
            T: ringbuf::traits::Consumer<Item = f32> + Send + 'static 
    {
        device.build_output_stream(stream_config, 
                                   move |data, _| {
                                       for sample in data.iter_mut() {
                                           *sample = consumer.try_pop().unwrap_or(0.0);
                                       }
                                   },
                                   |err| {
                                       panic!("Stream error: {:?}", err);
                                   }, 
                                   None)
                                   .expect("Failed to build output stream")
    }

    fn spawn_decode_thread<T>(mut decoder: AudioDecoder, mut producer: T)  -> JoinHandle<()>
        where
            T: ringbuf::traits::Producer<Item = f32> + Send + 'static
    {
        thread::spawn(move || {
            while let Some(sample) = decoder.get_next_sample() {
                for s in sample {
                    while producer.try_push(s).is_err() {
                        thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        })
    }
        
}
*/
