use engine::{
    ecs::{component::Component, Entity, WorldOp},
    graphics::{batch::Batch, common::RectF, texture::Texture},
};

use super::{approach, mover::Mover, position::Position};

pub struct Controller {
    pub width: u32,
    pub height: u32,
}

impl Controller {
    pub fn new(width: u32, height: u32) -> Self {
        Controller { width, height }
    }
}
impl Component for Controller {
    fn update<'a>(&mut self, entity: engine::ecs::Entity<'a, impl WorldOp>) {
        {
            let mut mover = entity.get_component::<Mover>().unwrap();
            let keyboard = engine::keyboard();
            if keyboard.keycodes.contains(&engine::Keycode::Left) {
                mover.speed.x -= 0.4f32;
            }
            if keyboard.keycodes.contains(&engine::Keycode::Right) {
                mover.speed.x += 0.4f32;
            }

            if keyboard.keycodes.contains(&engine::Keycode::Up) {
                mover.speed.y = 4f32;
            }

            // friction
            mover.speed.x = approach::<f32>(mover.speed.x, 0.0, 0.3);
            mover.speed.y = approach::<f32>(mover.speed.y, 0.0, 0.3);
        }
    }
}
