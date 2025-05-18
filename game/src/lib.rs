#![allow(warnings)]
mod components;
mod content;
mod game_state;
mod scene;
mod system;
mod target_manager;

extern crate engine;
extern crate nalgebra_glm as glm;
use content::Content;
use game_state::{GameState, SCREEN_HEIGHT, SCREEN_WIDTH};
use scene::GameScene;
use sdl2::{
    libc::{kevent, PF_KEY},
    AudioSubsystem, VideoSubsystem,
};

use common::{Debug, GameConfig, GameMemory, Keyboard, Mouse};
use components::{position::Position, room::Room};
use std::{env, mem::size_of};
use imgui::{Context, SuspendedContext, Ui};

// Pointer to the game memory (allocated in the main process — not in the dll)
// "static" is scoped to this dll instance. when hot-reloading a new dll this must be re-set
static mut MEMORY_PTR: *mut GameMemory = std::ptr::null_mut();

// Globally accessible utils
pub fn game_state() -> &'static mut GameState {
    unsafe { &mut *((*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState) }
}
fn current_scene() -> &'static mut GameScene {
    &mut game_state().scene_system.scene
}
fn current_room() -> &'static mut Room {
    let scene = current_scene();
    Content::map().get(scene.room_x as usize, scene.room_y as usize)
}

#[no_mangle]
pub extern "C" fn init(
    video_subsystem: &VideoSubsystem,
    audio_subsystem: &AudioSubsystem,
    game_memory_ptr: *mut GameMemory,
) {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
        MEMORY_PTR = game_memory_ptr; // get a pointer to the game memory
        Keyboard::init(&mut (*MEMORY_PTR).keyboard);
        Mouse::init(&mut (*MEMORY_PTR).mouse);
        Debug::init(&mut (*MEMORY_PTR).debug);
        engine::init(&video_subsystem, &audio_subsystem);

        if !(*MEMORY_PTR).initialized {
            let game_size = size_of::<GameState>(); // ~1232 bytes
            let available_memory = (*MEMORY_PTR).storage.len();
            assert!(
                game_size <= available_memory,
                "Game is too large for game_memory storage. Game size: {}, available mem: {}",
                game_size,
                available_memory
            );

            // storage is initialized with GameState and Content [ [GameState] [Content] ]
            let storage_ptr = (*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState;
            let content_ptr = storage_ptr.add(size_of::<GameState>()) as *mut Content;

            Content::load(content_ptr);
            storage_ptr.write(GameState::new()); // Directly write Game into storage

            (*MEMORY_PTR).initialized = true;
            game_state().init_systems();
        } else {
            let game: &mut GameState = &mut *((*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState);
            game.refresh();
        }
    }
}

#[no_mangle]
pub extern "C" fn get_config() -> GameConfig {
    GameConfig {
        window_width: SCREEN_WIDTH as u32,
        window_height: SCREEN_HEIGHT as u32,
    }
}

// TODO: pass a pointer to the keyboard instead — make it globally accessible throught the game
// instead of passing it around to every single function
#[no_mangle]
pub extern "C" fn update_game() {
    let game = game_state();
    game.update();
    game.render();
}

#[no_mangle]
pub extern "C" fn de_init() {
    // Called when the game lib is about to be dropped or reloaded
    unsafe {
        // This is a bit pointless sience the lib is getting destroyed but let's do it anyways
        MEMORY_PTR = std::ptr::null_mut();
    }
    // This does not delete the game memory — it only clears things in the game library itself
    // Mainly the static audio lib
    engine::deinit()
}

#[no_mangle]
pub extern "C" fn clear_game(game_memory: &mut GameMemory) {
    let game_ptr: *mut GameState = game_memory.storage.as_mut_ptr() as *mut GameState;
    unsafe {
        std::ptr::drop_in_place(game_ptr); // Drop Game manually
    }
    game_memory.initialized = false;
}
