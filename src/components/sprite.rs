use engine::{
    ecs::Component,
    graphics::{
        common::{PointF, RectF},
        texture::{SubTexture, Texture},
    },
};

use super::position::Position;

pub struct Sprite {
    pub subtexture: SubTexture,
}

impl Component for Sprite {
    fn render<'a>(
        &mut self,
        entity: engine::ecs::Entity<'a, impl engine::ecs::WorldOp>,
        batch: &mut engine::graphics::batch::Batch,
    ) {
        let position = entity
            .get_component::<Position>()
            .expect("Sprite component requires a Position component");

        let r = RectF {
            x:  position.x as f32,
            y: position.y as f32,
            w : self.subtexture.source.w as f32,
            h : self.subtexture.source.h as f32,
        };
        batch.sprite(&r, &self.subtexture, (0f32, 1f32, 1f32));
    }
}

impl Sprite {
    pub fn from_texture(texture: Texture) -> Self {
        Sprite {
            subtexture: SubTexture {
                texture,
                source: RectF {
                    x: 0f32,
                    y: 0f32,
                    w: texture.width as f32,
                    h: texture.height as f32,
                },
            },
        }
    }
    pub fn from_sub_texture(subtexture: SubTexture) -> Self {
        Sprite { subtexture }
    }
}
