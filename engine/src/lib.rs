#![deny(elided_lifetimes_in_paths)]
extern crate gl;
extern crate sdl2;
use imgui::Context;
use imgui_sdl2_support::SdlPlatform;

// todo: should this be pub?
pub mod graphics;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{Sdl, VideoSubsystem};
use std::time::Instant;

pub trait Game {
    fn init(&mut self);
    fn update(&mut self);
    fn render<'b>(&'b self, batch: &mut graphics::batch::Batch<'b>);
}

const FPS: u32 = 1_000_000_000u32 / 60;

pub fn start(mut game: impl Game) {
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
    let material = graphics::material::Material::new(shader);
    let mut mesh = graphics::mesh::Mesh::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    /* create context */
    let mut imgui = Context::create();
    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    /* create platform and renderer */
    let mut platform = SdlPlatform::init(&mut imgui);
    let imgui_renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video_subsystem.gl_get_proc_address(s) as _);

    unsafe {
        // todo: disable
        gl::Disable(gl::CULL_FACE);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }
    'running: loop {
        for event in event_pump.poll_iter() {
            platform.handle_event(&mut imgui, &event);
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        let start = Instant::now();
        {
            game.update();
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            // Todo: understand why this workaround works but owning vertices and indices inside the batch doesn't.
            let mut batch =
                graphics::batch::Batch::new(&mut mesh, &material, &mut vertices, &mut indices);
            batch.init();
            game.render(&mut batch);
            batch.clear();

            { // :::: Imgui :::::
                /* call prepare_frame before calling imgui.new_frame() */
                platform.prepare_frame(&mut imgui, &window, &event_pump);

                let ui = imgui.new_frame();
                /* create imgui UI here */
                ui.show_demo_window(&mut true);

                /* render imgui */
                imgui_renderer.render(&mut imgui)
            }

            window.gl_swap_window();
        }
        let delta = start.elapsed();
        println!("frame took: {}ms", delta.as_millis());
        let sleep_for = if delta.as_nanos() as u32 <= FPS {
            FPS - delta.as_nanos() as u32
        } else {
            // todo!: panic only in debug? maybe add a tolerance..
            // panic!("Game running too slow! delta: {}ms", delta.as_millis());
            0
        };
        println!("sleeping for remaining: {}ms", sleep_for / 1000000);
        ::std::thread::sleep(::std::time::Duration::new(0, sleep_for));
    }
}
