use engine::ecs::{World, WorldOp};

use crate::components::sprite::Sprite;

pub struct AnimationSystem;
impl AnimationSystem {
    pub fn tick(world: &World) {
        for sprite_entity in world.all_with::<Sprite>() {
            sprite_entity.get::<Sprite>().tick();
        }
    }
}
