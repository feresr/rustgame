use engine::{
    ecs::{Component, Entity, WorldOp},
    graphics::{batch::Batch, common::RectF, texture::Texture},
};

use super::{approach, mover::Mover, position::Position};

pub struct Player {
    pub width: u32,
    pub height: u32,
    pub texture: Texture,
    pub rect: RectF,
}

impl Player {
    pub fn new(width: u32, height: u32, texture: Texture) -> Self {
        Player {
            width,
            height,
            texture,
            rect: RectF::with_size(width as f32, height as f32),
        }
    }
}
impl Component for Player {
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

    fn render<'a>(&mut self, entity: Entity<'a, impl WorldOp>, batch: &mut Batch) {
        let position = entity.get_component::<Position>().unwrap();
        self.rect.x = position.x as f32;
        self.rect.y = position.y as f32;
        batch.tex(&self.rect, &self.texture, (0f32, 0f32, 0f32));
    }
}
