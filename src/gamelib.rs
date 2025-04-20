use std::{ffi::CString, path::PathBuf};

use common::{GameConfig, GameMemory};
use sdl2::{
    sys::{SDL_LoadFunction, SDL_LoadObject, SDL_UnloadObject},
    AudioSubsystem, VideoSubsystem,
};

pub type GetConfigFn = extern "C" fn() -> GameConfig;
pub type UpdateGameFunc = extern "C" fn();
pub type ClearGameMemFn = extern "C" fn(game_mmory: &mut GameMemory);
pub type InitGameFunc = extern "C" fn(
    video_subsystem: &VideoSubsystem,
    audio_subsystem: &AudioSubsystem,
    game_memory: *mut GameMemory,
);
pub type DeInitGameFunc = extern "C" fn();

pub struct GameLib {
    pub handle: *mut core::ffi::c_void,
    pub get_config: GetConfigFn,
    pub update: UpdateGameFunc,
    pub init: InitGameFunc,
    pub clear_game_mem: ClearGameMemFn,
    pub de_init: DeInitGameFunc,
}

impl GameLib {
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let c_path =
            CString::new(path.to_str().ok_or("Invalid path")?).map_err(|e| e.to_string())?;
        let handle = unsafe { SDL_LoadObject(c_path.as_ptr()) };

        if handle.is_null() {
            return Err("Failed to load library".to_string());
        }

        let func_name = CString::new("get_config").map_err(|e| e.to_string())?;
        let get_config: GetConfigFn = unsafe {
            let symbol = SDL_LoadFunction(handle, func_name.as_ptr());
            if symbol.is_null() {
                SDL_UnloadObject(handle);
                return Err("Failed to get_config function".to_string());
            }
            std::mem::transmute(symbol)
        };
        let func_name = CString::new("update_game").map_err(|e| e.to_string())?;
        let update: UpdateGameFunc = unsafe {
            let symbol = SDL_LoadFunction(handle, func_name.as_ptr());
            if symbol.is_null() {
                SDL_UnloadObject(handle);
                return Err("Failed to load function".to_string());
            }
            std::mem::transmute(symbol)
        };
        let func_name = CString::new("init").map_err(|e| e.to_string())?;
        let init: InitGameFunc = unsafe {
            let symbol = SDL_LoadFunction(handle, func_name.as_ptr());
            if symbol.is_null() {
                SDL_UnloadObject(handle);
                return Err("Failed to load init function".to_string());
            }
            std::mem::transmute(symbol)
        };
        let func_name = CString::new("clear_game").map_err(|e| e.to_string())?;
        let clear_game: ClearGameMemFn = unsafe {
            let symbol = SDL_LoadFunction(handle, func_name.as_ptr());
            if symbol.is_null() {
                SDL_UnloadObject(handle);
                return Err("Failed to load clear_game function".to_string());
            }
            std::mem::transmute(symbol)
        };
        let func_name = CString::new("de_init").map_err(|e| e.to_string())?;
        let de_init: DeInitGameFunc = unsafe {
            let symbol = SDL_LoadFunction(handle, func_name.as_ptr());
            if symbol.is_null() {
                SDL_UnloadObject(handle);
                return Err("Failed to load de_init function".to_string());
            }
            std::mem::transmute(symbol)
        };

        Ok(Self {
            handle,
            get_config,
            update,
            init,
            clear_game_mem: clear_game,
            de_init,
        })
    }
}

impl Drop for GameLib {
    fn drop(&mut self) {
        println!("calling de init for game lib");
        if !self.handle.is_null() {
            println!("invoked");
            (self.de_init)();
            unsafe { SDL_UnloadObject(self.handle) };
        }
    }
}
