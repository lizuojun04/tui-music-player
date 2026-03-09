use symphonia::core::{
    formats::{FormatReader, SeekMode, SeekTo},
    codecs::Decoder,
    io::MediaSourceStream,
    probe::Hint,
    audio::SampleBuffer,
    units::Time,
    meta,
};
use std::{
    path::{Path, PathBuf},
    fs::File
};
use crate::utils::utils;


pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    n_frames: u64,
    sample_rate: u32,
    channels: u16,
    first_frame: Option<Vec<f32>>
}

impl AudioDecoder {
    pub fn new(file_path: PathBuf) -> Self {
        let file = File::open(file_path).expect("Failed to open audio file");
        let media_source_stream = MediaSourceStream::new(Box::new(file), 
                                                                            Default::default());

        let hint = Hint::new();

        let probe = symphonia::default::get_probe()
            .format(&hint, 
                    media_source_stream, 
                    &Default::default(), 
                    &Default::default())
            .expect("Failed to probe audio format");

        let mut format_reader = probe.format;


        let track = format_reader
            .tracks()
            .iter()
            .find(Self::is_audio)
            .expect("No supported audio tracks found");
        let track_id = track.id;

        let n_frames = track.codec_params.n_frames.unwrap_or(0);

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .expect("Failed to create audio decoder");

        let (sample_rate, channels, first_frame) = Self::get_first_simple(&mut format_reader, &mut decoder, &track_id);

        Self {
            format_reader,
            decoder,
            track_id,
            n_frames,
            sample_rate,
            channels,
            first_frame
        }

    }

    pub fn get_song_info(&mut self, file_path: &Path) -> (String, String, String, f64) {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
        let (mut title, mut artist, mut album) = utils::parse_file_name(&file_name);

        if let Some(metadata) = self.format_reader.metadata().current() {
            for tag in metadata.tags() {
                match tag.std_key {
                    Some(meta::StandardTagKey::TrackTitle) => title = tag.value.to_string(),
                    Some(meta::StandardTagKey::Artist) => artist = tag.value.to_string(),
                    Some(meta::StandardTagKey::Album) => album = tag.value.to_string(),
                    _ => {}
                }
            }
        }

        (title, artist, album, self.n_frames as f64 / self.sample_rate as f64)
    }

    pub fn get_next_sample(&mut self) -> Option<Vec<f32>> {
        if let Some(frame) = self.first_frame.take() {
            return Some(frame);
        }
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

    fn get_first_simple(format_reader: &mut Box<dyn FormatReader>, decoder: &mut Box<dyn Decoder>, track_id: &u32) -> (u32, u16, Option<Vec<f32>>) {
        loop {
            let packet = match format_reader.next_packet() {
                Ok(packet) => packet,
                Err(_) => return (0, 0, None)
            };

            if packet.track_id() != *track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                        let sample_rate = decoded.spec().rate;
                        let channels = decoded.spec().channels.count() as u16;

                        let mut sample_buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());

                        sample_buffer.copy_interleaved_ref(decoded);

                        return (sample_rate, channels, Some(sample_buffer.samples().to_vec()));
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

    pub fn seek(&mut self, secs: u64) -> bool {
        let seek_to = SeekTo::Time {
            time: Time::new(secs, 0.0),
            track_id: Some(self.track_id)
        };

        match self.format_reader.seek(SeekMode::Coarse, seek_to) {
            Ok(_) => {
                self.decoder.reset();
                true
            },
            Err(_) => false
        }
    }

    fn is_audio(&track: &&symphonia::core::formats::Track) -> bool {
        let is_known_codec = track.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL;
        let has_audio_params = track.codec_params.sample_rate.is_some();
        is_known_codec && has_audio_params
    }
}
