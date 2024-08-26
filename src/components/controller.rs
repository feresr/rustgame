use engine::ecs::{component::Component, WorldOp};
use glm::clamp;

use super::{
    approach,
    collider::{Collider, Direction},
    mover::Mover,
};

#[derive(Clone, Copy)]
pub struct Controller {
    pub width: u32,
    pub height: u32,
    pub in_air: bool,
}

impl Controller {
    pub fn new(width: u32, height: u32) -> Self {
        Controller {
            width,
            height,
            in_air: false,
        }
    }
}
impl Component for Controller {}
