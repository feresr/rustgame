use engine::ecs::{component::Component, WorldOp};
use glm::clamp;

use super::{
    approach,
    collider::{Collider, Direction},
    mover::Mover,
};

#[derive(Clone, Copy)]
pub struct Player {
    pub width: u32,
    pub height: u32,
    pub in_air: bool,
    pub attack_timer: u32,
}

impl Player {
    pub fn new(width: u32, height: u32) -> Self {
        Player {
            width,
            height,
            in_air: false,
            attack_timer: 0,
        }
    }
    pub fn update(&mut self) {
        if self.attack_timer > 0 {
            self.attack_timer -= 1;
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
