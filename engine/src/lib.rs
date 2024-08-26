#![deny(elided_lifetimes_in_paths)]
extern crate gl;
extern crate nalgebra_glm as glm;
extern crate sdl2;

use graphics::batch::{Batch, ImGuiable};
use imgui::{Context, Ui};

pub mod ecs;
pub mod graphics;

use sdl2::event::Event;
pub use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{Sdl, VideoSubsystem};
use std::collections::HashSet;
use std::env;
use std::time::{Duration, Instant};

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / FPS);

pub struct Config {
    pub window_width: u32,
    pub window_height: u32,
}

pub trait Game {
    fn config(&self) -> Config;
    fn init(&mut self);
    fn update(&mut self) -> bool;
    fn render(&self, batch: &mut Batch);
    fn debug(&self, imgui: &Ui);
    fn dispose(&mut self);
}

static mut KEYBOARD: Option<Keyboard> = None;

pub fn keyboard() -> &'static mut Keyboard {
    unsafe {
        if KEYBOARD.is_none() {
            KEYBOARD = Some(Keyboard::new());
        }
        KEYBOARD.as_mut().unwrap()
    }
}

pub fn run(mut game: impl Game) {
    env::set_var("RUST_BACKTRACE", "1");
    // From: https://github.com/Rust-SDL2/rust-sdl2#use-opengl-calls-manually
    let config = game.config();
    let window_size = (config.window_width, config.window_height);
    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Window", window_size.0, window_size.1)
        .allow_highdpi()
        .opengl()
        // .borderless()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    /* create context */
    let mut imgui = Context::create();
    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    let mut platform = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let shader = graphics::shader::Shader::new(
        graphics::VERTEX_SHADER_SOURCE,
        graphics::FRAGMENT_SHADER_SOURCE,
    );
    let material = graphics::material::Material::new(shader);
    let mesh = graphics::mesh::Mesh::new();
    let mut batch = graphics::batch::Batch::new(mesh, material);
    let mut events = sdl_context.event_pump().unwrap();

    // OpenGL config
    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        gl::Enable(gl::MULTISAMPLE);

        gl::Enable(gl::DEPTH_TEST);
        // For equal z-index, do overwrite (default: g::LESS)
        gl::DepthFunc(gl::LEQUAL);

        gl::Enable(gl::BLEND);
        gl::BlendEquation(gl::FUNC_ADD);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    game.init();

    'game_loop: loop {
        let start = Instant::now();
        let keyboard = keyboard();

        let keys: HashSet<Keycode> = events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        keyboard.keycodes = keys;

        for ref event in events.poll_iter() {
            platform.handle_event(&mut imgui, event);
            if platform.ignore_event(&event) {
                continue;
            }
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'game_loop;
                }
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => {
                    keyboard.keycodes.insert(kc.to_owned());
                }
                _ => {}
            }
        }

        platform.prepare_frame(imgui.io_mut(), &window, &events.mouse_state());

        // Update
        game.update();
        // Render
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        batch.clear();
        game.render(&mut batch);

        unsafe {
            let mut error: u32 = gl::GetError();
            while error != gl::NO_ERROR {
                println!("GL error - {}", error);
                error = gl::GetError();
            }
        }

        // Imgui
        let frame_rate = imgui.io().framerate;
        let ui = imgui.frame();
        platform.prepare_render(&ui, &window);
        ui.window("Render calls")
            .size([400.0, 600.0], imgui::Condition::Appearing)
            .collapsed(true, imgui::Condition::Appearing)
            .build(|| {
                ui.text(format!(
                    "Frame took: {} milli-seconds",
                    start.elapsed().as_millis()
                ));
                ui.text(format!("Framerate: {} milli-seconds", frame_rate));

                // if cfg!(debug_assertions) {
                batch.render_imgui(ui);
                game.debug(ui)
                // }
            });
        platform.prepare_render(&ui, &window);
        renderer.render(&mut imgui);

        // println!("elapsed {:?}", start.elapsed());

        window.gl_swap_window();
        let sleep_until = start + FRAME_DURATION;
        while Instant::now() < sleep_until {
            // sleep
        }
        // let elapsed = start.elapsed();
        // if elapsed < FRAME_DURATION {
        //     let sleep_until = FRAME_DURATION - elapsed
        //     sleep();
        // }
        // TODO: Thread sleep might not be the right thing to do (imprecise - might sleep longer)
        // let sleep_for = if delta.as_nanos() as u32 <= FPS {
        //     FPS - delta.as_nanos() as u32
        // } else {
        //     // todo!: panic only in debug? maybe add a tolerance..
        //     // panic!("Game running too slow! delta: {}ms", delta.as_millis());
        //     0
        // };
        // // println!("sleeping for remaining: {}ms", sleep_for / 1000000);
        // // TODO: look into why imgui reports 30fps (and is probably right)

        // ::std::thread::sleep(::std::time::Duration::new(0, sleep_for));
    }
    game.dispose();
}

pub struct DebugOptions {
    pub cube_size: f32,
    pub camera_pos: [f32; 3],
    pub camera_target: [f32; 3],
    pub perspective: bool,
    pub fov: f32,
    pub pause: bool,
    pub render_cube_1: bool,
    pub render_cube_2: bool,
    pub render_background: bool,
}
pub struct Mouse {
    pub positon: (i32, i32),
    pub change: (i32, i32),
    pub pressing: bool,
}

pub struct Keyboard {
    pub keycodes: HashSet<Keycode>,
}
impl Keyboard {
    fn new() -> Self {
        Keyboard {
            keycodes: HashSet::new(),
        }
    }
}
