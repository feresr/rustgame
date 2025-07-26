#![allow(warnings)]

mod gamelib;

use common::{Debug, GameMemory, Keyboard, Mouse};
use gamelib::GameLib;
use imgui::sys::{igGetCurrentContext, igSetAllocatorFunctions, igSetCurrentContext, ImGuiMemAllocFunc, ImGuiStorage_SetAllInt};
use imgui::{Context, SuspendedContext};
use notify::{Config, Error, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::unsync::Lazy;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::libc::kill;
use sdl2::video::GLProfile;
use sdl2::{AudioSubsystem, Sdl, VideoSubsystem};
use std::collections::HashSet;
use std::env;
use std::f64::consts::PI;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use sdl2::sys::SDL_RenderPresent;

pub const FPS: u64 = 60;
pub const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / FPS);

fn get_lib_path() -> PathBuf {
    let mut path = PathBuf::from("./target/debug/libgame");

    if cfg!(target_os = "windows") {
        path.set_extension("dll");
    } else if cfg!(target_os = "macos") {
        path.set_extension("dylib");
    } else {
        path.set_extension("so");
    }
    path
}

/**
 * Watches the game library for updates
 */
fn check_for_updates_non_blocking(rx: &Receiver<Result<notify::Event, Error>>) -> bool {
    let mut updated = false;
    // Consume all existing file change events, check if at least one event happened
    while let Ok(_event) = rx.try_recv() {
        updated = true;
    }
    updated
}

fn main() {
    // Create sdl_first, it should be the last thing that gets dropped
    let sdl_context: Sdl = sdl2::init().unwrap();

    // TODO
    env::set_var("RUST_BACKTRACE", "1");

    // Load Game lib
    let lib_path = get_lib_path();
    let mut game = GameLib::load(&lib_path).unwrap();

    // Watch for game lib updates
    // TODO: debug only
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher
        .watch(&lib_path, RecursiveMode::NonRecursive)
        .unwrap();

    let mut config = (game.get_config)();
    let window_size = (config.window_width, config.window_height);
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();
    let audio_subsystem: AudioSubsystem = sdl_context.audio().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Window", window_size.0, window_size.1)
        // .allow_highdpi() TODO bring this back?
        .always_on_top()
        .opengl()
        // .borderless()
        .build()
        .unwrap();

    // let drawable_size = window.drawable_size();
    // let mut screen = Target::screen(drawable_size.0 as i32, drawable_size.1 as i32);

    let _ctx = window.gl_create_context().unwrap();

    let mut imgui = Context::create();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    // let mut platform = SdlPlatform::init(&mut imgui);

    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut game_memory = GameMemory::default();

    let mut events = sdl_context.event_pump().unwrap();

    Keyboard::init(&mut game_memory.keyboard);
    Mouse::init(&mut game_memory.mouse);
    Debug::init(&mut game_memory.debug);

    (game.init)(&video_subsystem, &audio_subsystem, &mut game_memory);

    'game_loop: loop {
        // Reload game if needed
        if check_for_updates_non_blocking(&rx) {
            game = GameLib::load(&lib_path).unwrap();
            (game.init)(&video_subsystem, &audio_subsystem, &mut game_memory);
        }

        let start = Instant::now();

        Keyboard::clear_pressed();
        Mouse::clear();

        for ref event in events.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; }

            match event {
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => {
                    match win_event {
                        sdl2::event::WindowEvent::Resized(_w, _hh) => {
                            let _drawable_size = window.drawable_size();
                            // screen = Target::screen(drawable_size.0 as i32, drawable_size.1 as i32);
                        }
                        _ => {
                            // no op
                        }
                    }
                }
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'game_loop;
                }
                Event::KeyDown { keycode: Some(kc), ..  } =>  Keyboard::press(kc.clone()),
                Event::KeyUp { keycode: Some(kc), ..  } => Keyboard::release(&kc),
                Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                    sdl2::mouse::MouseButton::Left => {
                        Mouse::press_left();
                    }
                    sdl2::mouse::MouseButton::Right => {
                        Mouse::press_right();
                    }
                    _ => {}
                },
                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    sdl2::mouse::MouseButton::Left => {
                        Mouse::release_left();
                    }
                    sdl2::mouse::MouseButton::Right => {
                        Mouse::release_right();
                    }
                    _ => {}
                },
                Event::MouseWheel { x, y, .. } => {
                    Mouse::set_wheel(*x, *y);
                }
                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    let wheight = window_size.1 as i32;
                    Mouse::set_position(*x, wheight - *y, *xrel, *yrel * -1);
                }
                _ => {}
            }
        }

        (game.update)();
        // Update
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &events.mouse_state());

        let ui = imgui.frame();
        imgui_sdl2.prepare_render(&ui, &window);

        if !Debug::is_empty() {
            Debug::render(ui);
            renderer.render(&mut imgui);
        }
        window.gl_swap_window();
        Debug::clear();

        let sleep_until = start + FRAME_DURATION;
        while Instant::now() < sleep_until {
            // sleep
        }
    }
    (game.clear_game_mem)(&mut game_memory);
}
