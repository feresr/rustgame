use std::{cmp::min, fs::File};

use crate::sdl2::audio::AudioFormatNum;
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired},
    AudioSubsystem,
};
use vorbis_rs::VorbisDecoder;

pub struct AudioPlayer {
    music: AudioDevice<Mixer>,
    sound: AudioDevice<Mixer>,
}

impl AudioPlayer {
    pub fn new(audio_subsystem: &AudioSubsystem) -> Self {
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None,
        };
        let music_device = audio_subsystem
            .open_playback(None, &spec, |spec| Mixer {
                spec,
                track: None,
                position: 0,
                should_loop: true,
            })
            .expect("cannot open music playback");
        music_device.resume();

        // TODO: Mix sounds. Right now only playing one sound at a time is supported
        let sound_device = audio_subsystem
            .open_playback(None, &spec, |spec| Mixer {
                spec,
                track: None,
                position: 0,
                should_loop: false,
            })
            .unwrap();
        sound_device.resume();

        Self {
            music: music_device,
            sound: sound_device,
        }
    }

    pub fn stop_music(&mut self) {
        self.music.lock().track = None;
        self.music.pause();
    }

    pub fn pause_music(&mut self) {
        self.music.pause();
    }

    pub fn play_sound(&mut self, track: &'static AudioTrack) {
        let mut mixer = self.sound.lock();
        mixer.track = Some(track);
        mixer.position = 0;
        mixer.spec.freq = track.sampling_frequency as i32;
        drop(mixer);
        self.sound.resume();
    }

    pub fn play_music(&mut self, track: &'static AudioTrack) {
        let mut mixer = self.music.lock();
        mixer.track = Some(track);
        mixer.position = 0;
        mixer.spec.freq = track.sampling_frequency as i32;
        drop(mixer);
        match self.music.status() {
            sdl2::audio::AudioStatus::Stopped | sdl2::audio::AudioStatus::Paused => {
                self.music.resume();
            }
            sdl2::audio::AudioStatus::Playing => {
                // already playing, no op
            }
        }
    }
}

pub struct Mixer {
    spec: AudioSpec,
    track: Option<&'static AudioTrack>,
    position: usize,
    should_loop: bool,
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
        // assert!(channels == 2, "channels: {}", channels);
        while let Some(decoded_block) = decoder.decode_audio_block().map_err(|e| e.to_string())? {
            if channels == 2 {
                left.extend_from_slice(decoded_block.samples()[0]);
                right.extend_from_slice(decoded_block.samples()[1]);
            } else {
                left.extend_from_slice(decoded_block.samples()[0]);
                right.extend_from_slice(decoded_block.samples()[0]);
            }
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
            for out in data.iter_mut() {
                *out = Self::Channel::SILENCE;
            }
            return;
        }
        let track = self.track.as_mut().unwrap();
        // In an interleaved audio format, samples from all channels for a single point in time are grouped
        // together, then the next point in time, and so on.
        // Most audio hardware and APIs expect interleaved data. When you play this buffer,
        // the audio system knows how to separate and route each sample to the appropriate channel.
        for (index, out) in data.iter_mut().enumerate() {
            let sample = if index % 2 == 0 {
                &track.left
            } else {
                &track.right
            };
            let sample = sample[self.position];
            *out = sample;

            // Increment position after updating the last sample
            // TODO: channels might have different lengths?
            if index % 2 != 0 {
                self.position += 1;
                if self.position >= track.length() {
                    self.position = 0;
                    if !self.should_loop {
                        self.track = None;
                    }
                    break;
                }
            }
        }
    }
}
