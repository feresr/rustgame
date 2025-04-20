use sdl2::{keyboard::Keycode, libc::kevent};
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
    pub keyboard : Keyboard,
    pub mouse : Mouse,
    pub storage: [u8; GAME_MEMORY],
}

impl Default for GameMemory {
    fn default() -> Self {
        Self {
            initialized: false,
            keyboard: Keyboard::default(),
            mouse: Mouse::default(),
            storage: [0; GAME_MEMORY],
        }
    }
}

#[repr(C)]
pub struct GameConfig {
    pub window_width: u32,
    pub window_height: u32,
}


// There is two versions of the static keyboard struct
// once in the runtime and once in the game dll (statics are not shared!)
static mut KEYBOARD: *mut Keyboard = std::ptr::null_mut();
static mut MOUSE: *mut Mouse = std::ptr::null_mut();


#[repr(C)]
#[derive(Default)]
pub struct Mouse {
    pub position: (i32, i32),
    pub wheel: (i32, i32),
    pub position_rel: (i32, i32),
    pub left : bool,
    pub right : bool,
    pub left_held : bool,
    pub right_held : bool,
}

impl Mouse {
    // Both the game gll and the runtime need to call init 
    // The Keyboard struct lives in the runtime
    pub fn init(mouse : *mut Mouse) {
        unsafe {
            MOUSE = mouse; 
        }
    }
    fn get() -> &'static mut Mouse {
        unsafe { &mut *MOUSE } 
    }
    
    // Setters
    pub fn release_left() {
        Self::get().left = false; 
    }
    pub fn press_left() {
        Self::get().left = true;
    }
    pub fn hold_left() {
        dbg!("hold left");
        Self::get().left_held = true; 
    }
    
    pub fn press_right() {
        Self::get().right = true;
    }
    pub fn release_right() {
        Self::get().right = false; 
    }
    pub fn hold_right() {
       Self::get().right_held = true; 
    }
    
    // Getters
    pub fn left_pressed() -> bool {
        Self::get().left
    }
    pub fn left_held() -> bool {
        Self::get().left_held
    }
    pub fn right_pressed() -> bool {
        Self::get().right
    }
    pub fn right_held() -> bool {
        Self::get().right_held
    }
    
    pub fn position() -> (i32, i32) {
        Self::get().position
    }
    pub fn position_rel() -> (i32, i32) {
        Self::get().position_rel
    }
    pub fn set_wheel(x: i32, y: i32) {
        Self::get().wheel = (x, y);
    }
    pub fn wheel() -> (i32, i32) {
        Self::get().wheel
    }
    
    pub fn set_position(x: i32, y: i32, xrel: i32, yrel: i32) {
        Self::get().position = (x, y);
        Self::get().position_rel = (xrel, yrel);
    }
    pub fn clear() {
        Self::get().position_rel = (0, 0);
        Self::get().wheel = (0, 0);
        Self::get().left = false;
        Self::get().right = false;
        Self::get().left_held = false;
        Self::get().right_held = false;
    }
}

#[repr(C)]
pub struct Keyboard {
    pub held: HashSet<Keycode>,
    pub pressed: HashSet<Keycode>,
}

impl Keyboard {
    
    // Both the game gll and the runtime need to call init 
    // The Keyboard struct lives in the runtime
    pub fn init(keyboard : *mut Keyboard) {
        unsafe {
            KEYBOARD = keyboard; 
        }
    }
    
    fn get() -> &'static mut Keyboard {
        unsafe { &mut *KEYBOARD } 
    }
    
    pub fn clear_pressed() {
        Self::get().pressed.clear();
    }
    
    pub fn release(key: &Keycode) {
        Self::get().pressed.remove(key);
    }
   pub fn press(key: Keycode) {
       Self::get().pressed.insert(key);
   }
   
   pub fn pressed(key: Keycode) -> bool {
       Self::get().pressed.contains(&key)
   }

   pub fn hold(keys: HashSet<Keycode>) {
       Self::get().held.clear();
       Self::get().held = keys;
   }
   pub fn held(key: &Keycode) -> bool {
       Self::get().held.contains(key)
   }
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            held: HashSet::new(),
            pressed: HashSet::new(),
        }
    }
}
