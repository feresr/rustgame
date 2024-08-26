use engine::{
    ecs::{World, WorldOp},
    graphics::{batch::Batch, common::RectF},
};

use crate::components::{position::Position, room::Room, sprite::Sprite};

pub struct RenderSystem;
impl RenderSystem {
    pub fn render(&self, world: &World, batch: &mut Batch) {
        for sprite in world.find_all::<Sprite>() {
            let entity = sprite.entity_id;
            let sprite = sprite.component.borrow();
            let position = world
                .find_component::<Position>(entity)
                .expect("Sprite component requires a Position");

            let rect = RectF {
                x: position.x as f32,
                y: position.y as f32,
                w: sprite.subtexture.source.w as f32,
                h: sprite.subtexture.source.h as f32,
            };
            batch.sprite(&rect, &sprite.subtexture, (0f32, 1f32, 1f32));
        }

        for room in world.find_all::<Room>() {
            let entity = room.entity_id;
            let mut room = room.component.borrow_mut();
            if let None = room.texture {
                room.prerender();
            }

            let position = world
                .find_component::<Position>(entity)
                .expect("Sprite component requires a Position");
            let offset = glm::vec3(position.x as f32, position.y as f32, 0.0f32);
            batch.push_matrix(glm::Mat4::new_translation(&offset));
            batch.tex(&room.rect, &room.texture.unwrap(), (1.0, 1.0, 1.0));
            batch.pop_matrix();
        }
    }
}
