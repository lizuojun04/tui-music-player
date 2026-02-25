use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use ringbuf::{traits::Split, HeapRb};
use crate::audio::decoder::AudioDecoder;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

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

        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");

        let stream_config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default
        };

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
        device.build_output_stream(&stream_config, 
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
