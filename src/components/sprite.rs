use engine::{
    ecs::Component,
    graphics::{
        common::RectF,
        texture::{SubTexture, Texture},
    },
};

pub struct Sprite {
    pub subtexture: SubTexture,
}

impl Component for Sprite {}

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
