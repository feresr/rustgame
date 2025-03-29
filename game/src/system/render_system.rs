use std::cell::RefCell;

use engine::{
    ecs::{World, WorldOp},
    graphics::{
        self,
        batch::Batch,
        common::RectF,
        material::Material,
        target::Target,
        texture::{Texture, TextureFormat, TextureSampler},
    },
};
use glm::pi;

use crate::{
    components::{light::Light, player::Player, position::Position, room::Room, sprite::Sprite}, game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH},
};

use super::light_system;

pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            in vec4 a_color;\n
            in vec4 a_type;\n 
            layout(location = 0) out vec4 FragColor;\n

            uniform sampler2D u_color_texture;\n
            uniform sampler2D u_normal_texture;\n

            uniform vec2 u_light_position[8];\n

            uniform ivec2 u_resolution;\n

            void main()\n
            {\n

                vec4 color = texture(u_color_texture, TexCoord); \n
                vec3 normal = texture(u_normal_texture, TexCoord).xyz; \n

                // Start with a base color \n
                vec4 highlights = color;\n

                normal = normalize(normal * 2.0 - 1.0); \n
                normal = normal * vec3(1.0, -1.0, 1.0); \n
                for (int i = 0; i < 5; i++) { \n
                    // dist grows too much if the light is closer to gl_FragCoord\n
                    float dist = distance(u_light_position[i].xy , gl_FragCoord.xy); \n
                    dist = mix(4.0, 0.0, clamp(dist / 100.0, 0.2, 1.0));
                    // float dist = 1.0;
                    vec3 ray = normalize(vec3(u_light_position[i].xy - gl_FragCoord.xy, 1.0)); \n
                    float intensity = max(0.0, dot(ray, normal.xyz));  \n

                    highlights += (color * intensity * dist) / 5.0; \n
                } \n
                
                // FragColor = color + highlights; \n 
                // FragColor = color * highlights; \n 
                FragColor = highlights; \n 
                // FragColor = mix(color, highlights, 0.4); \n
            }";

#[allow(dead_code)]
pub struct RenderSystem {
    albedo: Texture,
    normal: Texture,
    target: Target,
    material: Material,
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

        let shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        let mut material = Material::with_sampler(shader, TextureSampler::nearest());
        let sampler = TextureSampler::nearest();
        material.set_sampler("u_color_texture", &sampler);
        material.set_sampler("u_normal_texture", &sampler);
        RenderSystem {
            albedo: target.attachments[0].clone(),
            normal: target.attachments[1].clone(),
            target,
            material: material,
        }
    }

    pub fn color(&self) -> &Texture {
        &self.albedo
    }

    pub fn render(&mut self, world: &World, batch: &mut Batch) {
        self.target.clear((0f32, 0f32, 0f32, 0f32));
        batch.clear();

        let mut positions: [f32; 10] = [0.0f32; 10];
        for room_entity in world.all_with::<Room>() {
            let mut room = room_entity.get::<Room>();
            if let None = room.albedo_texture {
                room.prerender();
            }
            for (i, light_entity) in world.all_with::<Light>().enumerate() {
                let position = light_entity.get::<Position>();
                positions[i * 2] = position.x as f32 - room.rect.x;
                positions[i * 2 + 1] = position.y as f32 - room.rect.y;
            }

            let material = &mut self.material;
            material.set_texture("u_color_texture", &room.albedo_texture.unwrap());
            material.set_texture("u_normal_texture", &room.normal_texture.unwrap());
            material.set_vector2f("u_light_position[0]", &positions);
            batch.push_material(&material);
            batch.rect(&room.rect, (1.0, 1.0, 1.0, 1.0));
            batch.pop_material()
        }

        let mut rect = RectF::default();
        for sprite_entity in world.all_with::<Sprite>() {
            let sprite = sprite_entity.get::<Sprite>();
            let position = sprite_entity.get::<Position>();

            let pivot = sprite.pivot();
            batch.push_matrix(glm::translate(
                &glm::identity(),
                &glm::vec3((position.x as i32) as f32, (position.y as i32) as f32, 0f32),
            ));
            batch.push_matrix(glm::scale(
                &glm::identity(),
                &glm::vec3((sprite.scale_x) as f32, (sprite.scale_y) as f32, 1f32),
            ));

            let subtexture = sprite.subtexture();
            rect.x = -pivot.0;
            rect.y = -pivot.1;
            rect.w = subtexture.source.w as f32;
            rect.h = subtexture.source.h as f32;

            // batch.circle(pivot, 15f32, 4, (1f32, 1f32, 1f32, 1f32));

            if sprite.flip_x {
                rect.x += rect.w;
                rect.w = -rect.w;
            }
            if sprite.flip_y {
                rect.y += rect.h;
                rect.h = -rect.h;
            }

            batch.sprite(&rect, subtexture, (1f32, 1f32, 1f32, 1f32));

            batch.pop_matrix();
            batch.pop_matrix();
        }
        // Only in debug
        // Collider::render(&world, batch);

        let room_entity = world.all_with::<Room>().next().expect("No Room present");
        let ortho = &room_entity.get::<Room>().world_ortho;
        batch.render(&self.target, ortho);
        batch.clear();
    }
}
