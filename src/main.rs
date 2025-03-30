mod gamelib;

use common::{GameMemory, Keyboard};
use gamelib::GameLib;
use notify::{Config, Error, RecommendedWatcher, RecursiveMode, Watcher};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::{AudioSubsystem, Sdl, VideoSubsystem};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

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

    let mut game_memory = GameMemory {
        initialized: false,
        storage: [0; 1024 * 2], // 2 Kb
    };

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

    let config = (game.get_config)();
    let window_size = (config.window_width, config.window_height);
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();
    let audio_subsystem: AudioSubsystem = sdl_context.audio().unwrap();

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

    // let drawable_size = window.drawable_size();
    // let mut screen = Target::screen(drawable_size.0 as i32, drawable_size.1 as i32);

    let _ctx = window.gl_create_context().unwrap();

    /* create context */
    // let mut imgui = Context::create();
    /* disable creation of files on disc */
    // imgui.set_ini_filename(None);
    // imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    // imgui
    //     .fonts()
    //     .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    // let mut platform = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    // let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
    //     video_subsystem.gl_get_proc_address(s) as _
    // });

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut events = sdl_context.event_pump().unwrap();

    (game.init)(&video_subsystem, &audio_subsystem, &mut game_memory);
    'game_loop: loop {
        // Reload game if needed
        if check_for_updates_non_blocking(&rx) {
            game = gamelib::GameLib::load(&lib_path).unwrap();
            (game.init)(&video_subsystem, &audio_subsystem, &mut game_memory);
        }

        let start = Instant::now();
        let mut keyboard = Keyboard::default();

        keyboard.pressed.clear();
        for ref event in events.poll_iter() {
            // platform.handle_event(&mut imgui, event);
            // if platform.ignore_event(&event) {
            //     continue;
            // }
            match event {
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => {
                    match win_event {
                        sdl2::event::WindowEvent::Resized(_w, _hh) => {
                            let drawable_size = window.drawable_size();
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
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => {
                    if !keyboard.held.contains(&kc) {
                        keyboard.pressed.insert(kc.to_owned());
                    }
                }
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => {
                    keyboard.pressed.remove(&kc);
                }
                _ => {}
            }
        }
        let keys: HashSet<Keycode> = events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        keyboard.held = keys;

        // platform.prepare_frame(imgui.io_mut(), &window, &events.mouse_state());

        // Update
        (game.update)(&mut game_memory, &keyboard);

        // Imgui
        // let frame_rate = imgui.io().framerate;
        // let ui = imgui.frame();
        // platform.prepare_render(&ui, &window);
        // ui.window("Render calls")
        //     .size([400.0, 600.0], imgui::Condition::Appearing)
        //     .collapsed(true, imgui::Condition::Appearing)
        //     .build(|| {
        //         ui.text(format!(
        //             "Frame took: {} milli-seconds",
        //             start.elapsed().as_millis()
        //         ));
        //         ui.text(format!("Framerate: {} milli-seconds", frame_rate));

        //         // if cfg!(debug_assertions) {
        //         batch.render_imgui(ui);
        //         game.debug(ui)
        //         // }
        //     });
        // platform.prepare_render(&ui, &window);
        // renderer.render(&mut imgui);
        // println!("elapsed {:?}", start.elapsed());
        window.gl_swap_window();
        let sleep_until = start + FRAME_DURATION;
        while Instant::now() < sleep_until {
            // sleep
        }
    }
    (game.clear_game_mem)(&mut game_memory);
}
