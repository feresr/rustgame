#![deny(elided_lifetimes_in_paths)]
extern crate gl;
extern crate sdl2;

// todo: should this be pub?
pub mod graphics;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{Sdl, VideoSubsystem};

pub trait Game {
    fn init(&self);
    fn update(&self, delta: f64);
    fn render(&mut self, batch: &mut graphics::batch::Batch<'_>);
}

pub fn start<T: Game>(mut game: T) {
    // From: https://github.com/Rust-SDL2/rust-sdl2#use-opengl-calls-manually
    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Window", 800, 600)
        .opengl()
        .build()
        .unwrap();

    // Unlike the other example above, nobody created a context for your window, so you need to create one.
    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut event_pump = sdl_context.event_pump().unwrap();

    game.init();
    let shader = graphics::shader::Shader::new(
        graphics::VERTEX_SHADER_SOURCE,
        graphics::FRAGMENT_SHADER_SOURCE,
    );
    let mut batch = graphics::batch::Batch::new(&shader);
    batch.init();
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        game.update(0.0);
        unsafe {
            gl::ClearColor(0.0, 0.3, 0.7, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        batch.clear();
        game.render(&mut batch);
        window.gl_swap_window();
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
