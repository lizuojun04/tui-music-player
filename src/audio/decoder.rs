use symphonia::core::formats::FormatReader;
use symphonia::core::codecs::Decoder;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::audio::SampleBuffer;
use std::path::PathBuf;
use std::fs::File;

pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u16
}

impl AudioDecoder {
    pub fn new(file_path: PathBuf) -> Self {
        let file = File::open(file_path).expect("Failed to open audio file");
        let media_source_stream = MediaSourceStream::new(Box::new(file), 
                                                                            Default::default());

        let hint = Hint::new();

        let probe = symphonia::default::get_probe().format(&hint, 
                                                                        media_source_stream, 
                                                                        &Default::default(), 
                                                                        &Default::default())
                                                                .expect("Failed to probe audio format");

        let format_reader = probe.format;

        let track = format_reader.tracks()
                                         .iter()
                                         .find(Self::is_audio)
                                         .expect("No supported audio tracks found");
        let track_id = track.id;
        let sample_rate = match track.codec_params.sample_rate {
            Some(rate) => rate,
            None => panic!("Audio track does not have a sample rate")
        };
        let channels = match track.codec_params.channels {
            Some(channel) => channel.count() as u16,
            None => panic!("Audio track does not have a channel count")
        };

        let decoder = symphonia::default::get_codecs().make(&track.codec_params, &Default::default())
                                                                        .expect("Failed to create audio decoder");

        Self {
            format_reader,
            decoder,
            track_id,
            sample_rate,
            channels
        }

    }

    pub fn get_next_sample(&mut self) -> Option<Vec<f32>> {
        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(packet) => packet,
                Err(_) => return None
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                        let mut sample_buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());

                        sample_buffer.copy_interleaved_ref(decoded);

                        return Some(sample_buffer.samples().to_vec());
                },
                Err(_) => continue
            }
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_channels(&self) -> u16 {
        self.channels
    }

    fn is_audio(&track: &&symphonia::core::formats::Track) -> bool {
        let is_known_codec = track.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL;
        let has_audio_params = track.codec_params.sample_rate.is_some();
        is_known_codec && has_audio_params
    }
}
