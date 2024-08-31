use engine::ecs::{World, WorldOp};

use crate::components::sprite::Sprite;

pub struct AnimationSystem;
impl AnimationSystem {
    pub fn tick(&self, world: &World) {
        for sprite in world.find_all::<Sprite>() {
            sprite.component.borrow_mut().tick();
        }
    }
}
