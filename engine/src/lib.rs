#![deny(elided_lifetimes_in_paths)]
extern crate gl;
extern crate nalgebra_glm as glm;
extern crate sdl2;
use bevy_ecs::prelude::*;

use bevy_ecs::world::World;
use graphics::batch::{Batch, ImGuiable};
use imgui::{Context};
use imgui_sdl2::ImguiSdl2;

// todo: should this be pub?
pub mod graphics;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLProfile};
use sdl2::{EventPump, Sdl, VideoSubsystem};
use std::time::Instant;

#[derive(Component)]
struct Exit;

const FPS: u32 = 1_000_000_000u32 / 60;

pub fn start(init: &dyn Fn(&mut World, &mut Schedule, &mut Schedule) -> ()) {
    // From: https://github.com/Rust-SDL2/rust-sdl2#use-opengl-calls-manually
    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Window", 1400, 800)
        .allow_highdpi()
        .opengl()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut world = World::new();

    let shader = graphics::shader::Shader::new(
        graphics::VERTEX_SHADER_SOURCE,
        graphics::FRAGMENT_SHADER_SOURCE,
    );
    let material = graphics::material::Material::new(shader);
    let mesh = graphics::mesh::Mesh::new();
    let batch = graphics::batch::Batch::new(mesh, material);

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
    let platform = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });

    let event_pump = sdl_context.event_pump().unwrap();
    let mouse = Mouse {
        positon: (0, 0),
        change: (0, 0),
        pressing: false,
    };
    let keyboard = Keyboard { shift: false };

    world.insert_non_send_resource(imgui);
    world.insert_non_send_resource(event_pump);
    world.insert_non_send_resource(platform);
    world.insert_non_send_resource(renderer);
    world.insert_non_send_resource(window);
    world.insert_non_send_resource(batch);
    world.insert_non_send_resource(mouse);
    world.insert_non_send_resource(keyboard);

    // Create a new Schedule, which defines an execution strategy for Systems
    let mut update_schedule = Schedule::default();
    let mut render_schedule = Schedule::default();

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Enable(gl::MULTISAMPLE);
    }

    render_schedule.add_systems(swap_window);
    init(&mut world, &mut update_schedule, &mut render_schedule);
    render_schedule.add_systems(imgui_system);

    loop {
        let start = Instant::now();
        {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            update_schedule.run(&mut world);
            render_schedule.run(&mut world);

            // todo: this might not be the best way to query for a single item
            let exit = world.query::<(Entity, &Exit)>().iter(&world).count();
            if exit > 0 {
                break;
            }
        }
        let delta = start.elapsed();
        println!("frame total took: {}ms", delta.as_millis());
        let sleep_for = if delta.as_nanos() as u32 <= FPS {
            FPS - delta.as_nanos() as u32
        } else {
            // todo!: panic only in debug? maybe add a tolerance..
            // panic!("Game running too slow! delta: {}ms", delta.as_millis());
            0
        };
        println!("sleeping for remaining: {}ms", sleep_for / 1000000);
        // TODO: look into why imgui reports 30fps (and is probably right)
        ::std::thread::sleep(::std::time::Duration::new(0, sleep_for / 2));
    }
}

#[derive(Component)]
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
    pub shift: bool,
}

fn imgui_system(
    mut imgui: NonSendMut<'_, Context>,
    mut batch: NonSendMut<'_, Batch>,
    mut mouse: NonSendMut<'_, Mouse>,
    mut keyboard: NonSendMut<'_, Keyboard>,
    mut event_pump: NonSendMut<'_, EventPump>,
    mut platform: NonSendMut<'_, ImguiSdl2>,
    mut debug_options: Query<'_, '_, &mut DebugOptions>,
    window: NonSend<'_, sdl2::video::Window>,
    renderer: NonSend<'_, imgui_opengl_renderer::Renderer>,
    mut commands: Commands<'_, '_>,
) {
    mouse.change = (0, 0);
    for ref event in event_pump.poll_iter() {
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
                commands.spawn(Exit {});
                return;
            }
            Event::KeyDown {
                keycode: Some(Keycode::LShift),
                ..
            } => {
                keyboard.shift = true;
            }
            Event::KeyUp {
                keycode: Some(Keycode::LShift),
                ..
            } => {
                keyboard.shift = false;
            }
            Event::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                mouse.positon = (*x, *y);
                mouse.change = (*xrel, *yrel);
            }
            Event::MouseButtonDown { .. } => {
                mouse.pressing = true;
            }
            Event::MouseButtonUp { .. } => {
                mouse.pressing = false;
            }
            _ => {}
        }
    }

    platform.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
    let ui = imgui.frame();

    for mut slider in &mut debug_options {
        ui.input_float3("camera_pos", &mut slider.camera_pos)
            .build();
        ui.input_float3("camera_target", &mut slider.camera_target)
            .build();
        ui.slider("size", -5.0, 5.0, &mut slider.cube_size);
        ui.checkbox("perspective", &mut slider.perspective);
        ui.checkbox("pause", &mut slider.pause);
        ui.slider("fov", 0.0, 4.0, &mut slider.fov);
        ui.text(format!(
            "Mouse {:?}, clicking= {}, change= {:?}",
            mouse.positon, mouse.pressing, mouse.change
        ));
        ui.checkbox("cube 1", &mut slider.render_cube_1);
        ui.checkbox("cube 2", &mut slider.render_cube_2);
        ui.checkbox("background", &mut slider.render_background);
    }

    batch.render_imgui(ui);

    platform.prepare_render(&ui, &window);
    renderer.render(&mut imgui);
}

fn swap_window(window: NonSend<'_, sdl2::video::Window>) {
    window.gl_swap_window();
}
