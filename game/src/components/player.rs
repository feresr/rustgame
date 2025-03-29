use engine::ecs::component::Component;

#[derive(Clone, Copy, Default)]
pub struct Player {
    pub in_air: bool,
    pub was_in_air: bool,
    pub attack_timer: u16,
    pub jump_buffer: u8,
    pub coyote_buffer: u8,
}

pub const JUMP_BUFFER_TIME: u8 = 8;
pub const COYOTE_BUFFER_TIME: u8 = 4;
pub const JUMP_SPEED: f32 = 10f32;
pub const WALK_SPEED: f32 = 0.6f32;

impl Player {
    pub fn update(&mut self) {
        if self.attack_timer > 0 {
            self.attack_timer -= 1;
        }
        if self.jump_buffer > 0 {
            self.jump_buffer -= 1;
        }
        if self.coyote_buffer > 0 {
            self.coyote_buffer -= 1;
        }
    }
    pub fn attack(&mut self) {
        if self.attack_timer == 0 {
            self.attack_timer = 30;
        }
    }
    pub fn is_attacking(&self) -> bool {
        self.attack_timer > 0
    }
}
impl Component for Player {}
