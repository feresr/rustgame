use std::{cmp::min, fs::File, io::BufReader};

use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    AudioSubsystem,
};
use vorbis_rs::{VorbisDecoder, VorbisEncoderBuilder};

pub struct AudioPlayer {
    device: AudioDevice<Mixer>,
}
impl AudioPlayer {
    pub fn new(audio_subsystem: AudioSubsystem) -> Self {
        let audio = AudioTrack::new("src/assets/song.ogg").unwrap();
        let spec = AudioSpecDesired {
            freq: Some(audio.sampling_frequency as i32),
            channels: Some(2),
            // The size of the audio buffer in samples
            samples: None,
        };
        let device = audio_subsystem
            .open_playback(None, &spec, |spec| Mixer {
                track: Some(audio),
                position: 0,
            })
            .unwrap();
        device.resume();
        Self { device }
    }

    pub fn update(&mut self) {
        let mut mixer = self.device.lock();
        if let Some(track) = &mixer.track {
            if mixer.position >= track.length() {
                mixer.position = 0;
                // self.device.pause()
            }
        }
    }

    pub fn play(&mut self, track: AudioTrack) {
        let mut mixer = self.device.lock();
        mixer.track = Some(track);
        mixer.position = 0;
    }
}

pub struct Mixer {
    track: Option<AudioTrack>,
    position: usize,
}

pub struct AudioTrack {
    left: Vec<f32>,
    right: Vec<f32>,
    pub sampling_frequency: u32,
}

impl AudioTrack {
    pub fn new(file_path: &str) -> Result<AudioTrack, String> {
        let mut file = File::open(file_path).map_err(|e| e.to_string())?;
        let mut decoder = VorbisDecoder::new(&mut file).map_err(|e| e.to_string())?;
        let mut left = Vec::new();
        let mut right = Vec::new();
        let freq = decoder.sampling_frequency().get();
        let channels = decoder.channels().get();
        assert!(channels == 2);
        while let Some(decoded_block) = decoder.decode_audio_block().map_err(|e| e.to_string())? {
            left.extend_from_slice(decoded_block.samples()[0]);
            right.extend_from_slice(decoded_block.samples()[1]);
        }
        Ok(Self {
            left,
            right,
            sampling_frequency: freq,
        })
    }
    pub fn length(&self) -> usize {
        min(self.left.len(), self.right.len())
    }
}

impl AudioCallback for Mixer {
    type Channel = f32;

    fn callback(&mut self, data: &mut [Self::Channel]) {
        if self.track.is_none() {
            return;
        }
        let track = self.track.as_mut().unwrap();
        // In an interleaved audio format, samples from all channels for a single point in time are grouped
        // together, then the next point in time, and so on.
        // Most audio hardware and APIs expect interleaved data. When you play this buffer,
        // the audio system knows how to separate and route each sample to the appropriate channel.
        println!("data: {:?}", data.len());
        for (index, out) in data.iter_mut().enumerate() {
            let sample = if index % 2 == 0 {
                &track.left
            } else {
                &track.right
            };
            let sample = sample[self.position];
            *out = sample;

            // Increment position after updating the last sample
            if index % 2 != 0 {
                self.position += 1;
                if self.position >= track.length() {
                    self.position = 0;
                    break;
                }
            }
        }
    }
}
