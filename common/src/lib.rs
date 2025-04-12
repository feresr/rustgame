use sdl2::keyboard::Keycode;
use std::collections::HashSet;

#[macro_export]
macro_rules! check_gl_errors {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            unsafe {
                while gl::GetError() != gl::NO_ERROR {
                    panic!("OpenGL error: {}", $msg);
                }
            }
        }
    };
}

// 4 Kb
const GAME_MEMORY: usize = 1024 * 8;

#[repr(C)]
pub struct GameMemory {
    pub initialized: bool,
    pub storage: [u8; GAME_MEMORY],
}

impl Default for GameMemory {
    fn default() -> Self {
        Self {
            initialized: false,
            storage: [0; GAME_MEMORY],
        }
    }
}

#[repr(C)]
pub struct GameConfig {
    pub window_width: u32,
    pub window_height: u32,
}

#[repr(C)]
pub struct Keyboard {
    pub held: HashSet<Keycode>,
    pub pressed: HashSet<Keycode>,
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            held: HashSet::new(),
            pressed: HashSet::new(),
        }
    }
}
