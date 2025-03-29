#![deny(elided_lifetimes_in_paths)]
extern crate gl;
extern crate nalgebra_glm as glm;
extern crate sdl2;

use audio::AudioPlayer;
use common::Keyboard;
use imgui::Ui;

pub mod audio;
pub mod ecs;
pub mod graphics;

pub use sdl2::keyboard::Keycode;
use sdl2::{AudioSubsystem, VideoSubsystem};
use std::time::Duration;

pub const FPS: u64 = 60;
pub const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / FPS);

pub struct Config {
    pub window_width: u32,
    pub window_height: u32,
}

pub static mut AUDIO: Option<AudioPlayer> = None;

pub fn audio() -> &'static mut AudioPlayer {
    unsafe { AUDIO.as_mut().unwrap() }
}

pub fn init(video_subsystem: &VideoSubsystem, audio_subsystem: &AudioSubsystem) {
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    unsafe {
        if AUDIO.is_none() {
            let audio_player = AudioPlayer::new(audio_subsystem);
            AUDIO = Some(audio_player)
        }
    }

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        gl::Enable(gl::MULTISAMPLE);
        gl::Enable(gl::DEPTH_TEST);
        // For equal z-index, do overwrite (default: g::LESS)
        gl::DepthFunc(gl::LEQUAL);
        gl::Disable(gl::STENCIL_TEST); // Stencil disabled by default
        gl::ClearStencil(0);
        gl::Enable(gl::BLEND);
    }
}

pub fn update() {
    unsafe {
        let mut error: u32 = gl::GetError();
        while error != gl::NO_ERROR {
            println!("GL error - {}", error);
            error = gl::GetError();
        }
    }
}
