mod aseprite;
mod components;
mod content;
mod game_state;
mod scene;
mod system;

extern crate engine;
extern crate nalgebra_glm as glm;
use game_state::{GameState, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::{AudioSubsystem, VideoSubsystem};

use common::{GameConfig, GameMemory, Keyboard};
use components::position::Position;
use content::content;
use std::{env, mem::size_of};

#[no_mangle]
pub extern "C" fn init(
    video_subsystem: &VideoSubsystem,
    audio_subsystem: &AudioSubsystem,
    game_memory: &mut GameMemory,
) {
    env::set_var("RUST_BACKTRACE", "1");
    engine::init(&video_subsystem, &audio_subsystem);
    if !game_memory.initialized {
        let game_size = size_of::<GameState>(); // ~1232 bytes
        let mem = game_memory.storage.len();
        assert!(
            game_size <= mem,
            "Game is too large for game_memory storage"
        );
        unsafe {
            let storage_ptr = game_memory.storage.as_mut_ptr() as *mut GameState;
            storage_ptr.write(GameState::new()); // Directly write Game into storage
        }
        game_memory.initialized = true;
    } else {
        let game: &mut GameState =
            unsafe { &mut *(game_memory.storage.as_mut_ptr() as *mut GameState) };
        game.refresh();
    }
}

#[no_mangle]
pub extern "C" fn get_config() -> GameConfig {
    return GameConfig {
        window_width: SCREEN_WIDTH as u32,
        window_height: SCREEN_HEIGHT as u32,
    };
}

#[no_mangle]
pub extern "C" fn update_game(game_memory: &mut GameMemory, keyboard: &Keyboard) {
    let game: &mut GameState =
        unsafe { &mut *(game_memory.storage.as_mut_ptr() as *mut GameState) };
    game.update(keyboard);
    game.render();
}

#[no_mangle]
pub extern "C" fn de_init() {
    // Called when the game lib is about to be dropped or reloaded
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
