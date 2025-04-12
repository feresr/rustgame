mod aseprite;
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
use sdl2::{AudioSubsystem, VideoSubsystem};

use common::{GameConfig, GameMemory, Keyboard};
use components::{position::Position, room::Room};
use std::{env, mem::size_of};

// Pointer to the game memory (allocated in the main process — not in the dll)
static mut MEMORY_PTR: *mut GameMemory = std::ptr::null_mut();

// Globally accessible utils
pub fn game_state() -> &'static mut GameState {
    return unsafe { &mut *((*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState) };
}
pub fn content() -> &'static mut Content {
    return unsafe {
        let storage_ptr = (*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState;
        let content = storage_ptr.add(size_of::<GameState>()) as *mut Content;
        &mut (*content)
    };
}
fn current_scene() -> &'static mut GameScene {
    return &mut game_state().scene_system.scene;
}
fn current_room() -> &'static mut Room {
    let scene = current_scene();
    content()
        .map
        .get(scene.room_x as usize, scene.room_y as usize)
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

            let storage_ptr = (*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState;

            dbg!(size_of::<GameState>());
            let content_ptr = storage_ptr.add(size_of::<GameState>()) as *mut Content;

            content_ptr.write(Content::load());
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
    return GameConfig {
        window_width: SCREEN_WIDTH as u32,
        window_height: SCREEN_HEIGHT as u32,
    };
}

// TODO: pass a pointer to the keyboard instead — make it globally accessible throught the game
// instead of passing it around to every single function
#[no_mangle]
pub extern "C" fn update_game(keyboard: &Keyboard) {
    let game = game_state();
    game.update(keyboard);
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
