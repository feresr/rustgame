use engine::{
    ecs::{World, WorldOp},
    graphics::{
        batch::Batch,
        common::RectF,
        target::Target,
        texture::{Texture, TextureFormat},
    },
};

use crate::{
    components::{position::Position, room::Room, sprite::Sprite},
    GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH,
};

pub struct RenderSystem {
    albedo: Texture,
    normal: Texture,
    target: Target,
}

impl RenderSystem {
    pub fn new() -> Self {
        let attachments = [
            // Albedo
            TextureFormat::RGBA,
            // Normal
            TextureFormat::RGBA,
        ];
        let target = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &attachments,
        );

        RenderSystem {
            albedo: target.attachments[0].clone(),
            normal: target.attachments[1].clone(),
            target,
        }
    }

    pub fn color(&self) -> &Texture {
        &self.albedo
    }

    pub fn render(&self, world: &World, batch: &mut Batch) {
        self.target.clear((0f32, 0f32, 0f32, 0f32));
        batch.clear();

        for room in world.find_all::<Room>() {
            let mut room = room.component.borrow_mut();
            if let None = room.texture {
                room.prerender();
            }
            batch.tex(&room.rect, &room.texture.unwrap(), (1.0, 1.0, 1.0, 1.0));
        }

        let mut rect = RectF::default();
        for sprite in world.find_all::<Sprite>() {
            let entity = sprite.entity_id;
            let sprite = sprite.component.borrow();
            let position = world
                .find_component::<Position>(entity)
                .expect("Sprite component requires a Position");

            let subtexture = sprite.subtexture();
            let pivot = sprite.pivot();
            rect.x = (position.x - pivot.0 as i32) as f32;
            rect.y = (position.y - pivot.1 as i32) as f32;
            rect.w = subtexture.source.w as f32;
            rect.h = subtexture.source.h as f32;

            if sprite.flip_x {
                rect.x += rect.w;
                rect.w = -rect.w;
            }
            if sprite.flip_y {
                rect.y += rect.h;
                rect.h = -rect.h;
            }

            batch.sprite(&rect, subtexture, (1f32, 1f32, 1f32, 1f32));
        }

        let room: &engine::ecs::ComponentWrapper<Room> =
            world.find_all::<Room>().next().expect("No Room present");
        let ortho = &room.component.borrow().world_ortho;
        batch.render(&self.target, ortho);
        batch.clear();
    }
}
