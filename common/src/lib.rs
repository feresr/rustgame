use imgui::{Context, Image, ImageButton, TextureId, Ui};
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
    pub keyboard: Keyboard,
    pub mouse: Mouse,
    pub debug: Debug,
    pub storage: [u8; GAME_MEMORY],
}

impl GameMemory {
    pub fn default() -> Self {
        Self {
            initialized: false,
            keyboard: Keyboard::default(),
            debug: Debug::default(),
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

// There is two versions of the static MOUSE/KEYBOARD struct
// once in the runtime and once in the game dll (statics are not shared!)
static mut KEYBOARD: *mut Keyboard = std::ptr::null_mut();
static mut MOUSE: *mut Mouse = std::ptr::null_mut();

static mut DEBUG: *mut Debug = std::ptr::null_mut();

enum UiElement {
    Text(String),
    Separator,
    Button(fn()),
    Checkbox(String, bool, Box<dyn Fn() -> ()>),
    SameLine,
    NewLine,
    Image(
        u32, // id
        usize,
        (f32, f32),           // Size of the image
        ([f32; 2], [f32; 2]), // UV coordinates
    ),
}

#[repr(C)]
#[derive(Default)]
pub struct DebugWindow {
    title: String,
    size: (f32, f32),
    items: Vec<UiElement>,
}

#[repr(C)]
#[derive(Default)]
pub struct Debug {
    pub windows: Vec<DebugWindow>,
    pub events: HashSet<u32>,
}
impl Debug {
    pub fn init(debug: *mut Debug) {
        unsafe {
            DEBUG = debug;
        }
    }
    pub fn get() -> &'static mut Debug {
        unsafe { &mut *DEBUG }
    }
    pub fn window(name: &str) {
        Self::get().windows.push(DebugWindow {
            title: name.to_string(),
            size: (300.0, 100.0),
            items: Vec::new(),
        });
    }
    pub fn window_size(name: &str, width: f32, height: f32) {
        Self::get().windows.push(DebugWindow {
            title: name.to_string(),
            size: (width, height),
            items: Vec::new(),
        });
    }
    pub fn button(f: fn()) {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::Button(f));
    }
    pub fn image(id: u32, textureId: usize, size: (f32, f32)) {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::Image(
            id,
            textureId,
            size,
            ([0.0, 0.0], [1.0, 1.0]),
        ));
    }
    pub fn same_line() {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::SameLine);
    }

    pub fn new_line() {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::NewLine);
    }

    pub fn sprite(id: u32, texture_id: usize, size: (f32, f32), uv: ([f32; 2], [f32; 2])) -> bool {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::Image(id, texture_id, size, uv));
        let events = &Self::get().events;
        return events.contains(&id);
    }

    pub fn checkbox(name: &str, value: bool, f: Box<dyn Fn() -> ()>) {
        let window = Self::get().windows.last_mut().unwrap();
        window
            .items
            .push(UiElement::Checkbox(name.to_string(), value, f));
    }

    pub fn display(item: &str) {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::Text(item.to_string()));
    }
    pub fn separator() {
        let window = Self::get().windows.last_mut().unwrap();
        window.items.push(UiElement::Separator);
    }
    pub fn is_empty() -> bool {
        Self::get().windows.is_empty()
    }
    pub fn clear() {
        Self::get().windows.clear();
    }
    pub fn render(ui: &Ui) {

        let debug = Debug::get();
        debug.events.clear();

        for window in Self::get().windows.iter_mut() {
            ui.window(window.title.as_str())
                .size(
                    [window.size.0, window.size.1],
                    imgui::Condition::FirstUseEver,
                )
                .build(|| {
                    for item in window.items.iter_mut() {
                        match item {
                            UiElement::Text(text) => {
                                ui.text(text.as_str());
                            }
                            UiElement::Separator => {
                                ui.separator();
                            }
                            UiElement::Button(f) => {
                                if ui.color_button("Click me", [1.0, 0.0, 0.0, 1.0]) {
                                    f()
                                }
                            }
                            UiElement::Checkbox(name, value, f) => {
                                if ui.checkbox(name, value) {
                                    f()
                                }
                            }
                            UiElement::Image(id, texture_id, size, uv) => {
                                if ui
                                    .image_button_config(
                                        id.to_string(),
                                        TextureId::new(*texture_id),
                                        [size.0, size.1],
                                    )
                                    .uv0(uv.0)
                                    .uv1(uv.1)
                                    .build()
                                {
                                    debug.events.insert(*id);
                                }
                            }
                            UiElement::SameLine => ui.same_line(),
                            UiElement::NewLine => ui.new_line(),
                        }
                    }
                });
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct Mouse {
    pub position: (i32, i32),
    pub wheel: (i32, i32),
    pub position_rel: (i32, i32),
    pub left: bool,
    pub right: bool,
    pub left_held: bool,
    pub right_held: bool,
}

impl Mouse {
    // Both the game dll and the runtime need to call init
    // The Keyboard struct lives in the runtime
    pub fn init(mouse: *mut Mouse) {
        unsafe {
            MOUSE = mouse;
        }
    }
    fn get() -> &'static mut Mouse {
        unsafe { &mut *MOUSE }
    }

    // Setters
    pub fn release_left() {
        let mouse = Self::get();
        mouse.left = false;
        mouse.left_held = false;
    }
    pub fn press_left() {
        let mouse = Self::get();
        mouse.left = !mouse.left_held; // left_held is false on the first iteration, true afterwards
        mouse.left_held = true;
    }

    pub fn press_right() {
        let mouse = Self::get();
        mouse.right = !mouse.right_held; // right_held is false on the first iteration, true afterwards
        mouse.right_held = true;
    }
    pub fn release_right() {
        let mouse = Self::get();
        mouse.right = false;
        mouse.right_held = false;
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
        Self::get().position_rel.0 += xrel;
        Self::get().position_rel.1 += yrel;
    }
    pub fn clear() {
        Self::get().position_rel = (0, 0);
        Self::get().wheel = (0, 0);
        // Self::get().left = false;
        // Self::get().right = false;
        // Self::get().left_held = false;
        // Self::get().right_held = false;
    }
}

#[repr(C)]
pub struct Keyboard {
    pub held: HashSet<Keycode>,
    pub pressed: HashSet<Keycode>,
}

impl Keyboard {
    // Both the game dll and the runtime need to call init
    // The Keyboard struct lives in the runtime
    pub fn init(keyboard: *mut Keyboard) {
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
