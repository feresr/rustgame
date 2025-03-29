use std::collections::HashSet;
use sdl2::keyboard::Keycode;

#[repr(C)]
pub struct GameMemory {
    pub initialized: bool,
    pub storage: [u8; 1024 * 2], // 2 Kb
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