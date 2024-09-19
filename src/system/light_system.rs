use std::cell::RefCell;

use engine::{
    ecs::{World, WorldOp},
    graphics::{
        self,
        batch::{Batch, Stencil},
        blend::{self},
        common::RectF,
        material::Material,
        target::Target,
        texture::{Texture, TextureFormat, TextureSampler},
    },
};

use crate::{
    components::{light::Light, position::Position, room::Room},
    GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, TILE_SIZE,
};
// todo a_type is (mult wash fill pad) document better
pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            in vec4 a_color;\n
            in vec4 a_type;\n 
            layout(location = 0) out vec4 FragColor;\n

            uniform vec2 u_light_position;\n
            uniform float u_light_radius;\n

            uniform sampler2D u_texture;\n
            uniform ivec2 u_resolution;\n

            void main()\n
            {\n
                vec2 frag_to_light = gl_FragCoord.xy - u_light_position; \n
                if (length(frag_to_light) > u_light_radius) {\n
                discard; \n
                } \n
                frag_to_light = normalize(frag_to_light);
                frag_to_light *= 0.5; 
                frag_to_light += vec2(1.0, 1.0); 
                FragColor = vec4(frag_to_light.xy, a_color.x, 1.0); \n
            }";

pub struct LightSystem {
    // We need to use a RefCell because we need to update the texture of the target
    // This is internally mutable. The API should not care about this impl detail.
    // Also by the end of the render function call, the state is back to its original state.
    target: RefCell<Target>,
    texture: Texture,
    material: RefCell<Material>,
}

impl LightSystem {
    pub fn new() -> Self {
        // TODO: could this be TextureFormat:R?
        let attachments = [TextureFormat::RGBA, TextureFormat::DepthStencil];
        let target = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &attachments,
        );
        let shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        let material = Material::with_sampler(shader, TextureSampler::nearest());

        LightSystem {
            texture: target.color().clone(),
            target: RefCell::new(target),
            material: RefCell::new(material),
        }
    }

    pub fn color(&self) -> &Texture {
        &self.texture
    }

    pub fn render(&self, world: &World, batch: &mut Batch) {
        let base_color = (0.0, 0.0, 0.0, 1.0);
        let light_color = (1.00, 1.00, 1.00, 1.0);
        const PROJECTION_DISTANCE: f32 = 140.0;

        let room: &engine::ecs::ComponentWrapper<Room> =
            world.find_all::<Room>().next().expect("No Room present");
        let room_position = world
            .find_component::<Position>(room.entity_id)
            .expect("Sprite component requires a Position");
        let material = self.material.borrow();
        let target = self.target.borrow_mut();

        // TODO: this is the camera. should the camera be part of the world (an entity)?
        let ortho = &room.component.borrow().world_ortho;
        target.clear(base_color);
        // Make the target non-drawable
        for light in world.find_all::<Light>() {
            batch.push_material(&material);
            target.clear_stencil(0);
            let id = light.entity_id;

            let light_position = world
                .find_component::<Position>(id)
                .expect("Light has no position")
                .as_vec2();

            // normalise light position
            let ligh_posx = light_position.x - room_position.x as f32;
            let ligh_posy = light_position.y - room_position.y as f32;
            material.set_value2f("u_light_position", (ligh_posx, ligh_posy));
            material.set_valuef("u_light_radius", PROJECTION_DISTANCE / 2f32);

            // Draw oclusion shadows (in the stencil buffer)
            batch.set_stencil(Stencil::write(1));
            for tile in room.component.borrow().layers.first().unwrap().tiles.iter() {
                let tile_position = glm::vec2(
                    room_position.x as f32 + tile.x as f32,
                    room_position.y as f32 + tile.y as f32,
                );

                let tile_light_distance = glm::distance(&tile_position, &light_position);
                if tile_light_distance > PROJECTION_DISTANCE {
                    continue;
                }

                let x1 = tile_position.x;
                let y1 = tile_position.y;
                let x2 = x1 + TILE_SIZE as f32;
                let y2 = y1 + TILE_SIZE as f32;

                let mut points = vec![
                    glm::vec2(x1, y1),
                    glm::vec2(x2, y1),
                    glm::vec2(x2, y2),
                    glm::vec2(x1, y2),
                ];
                points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

                if light_position.x > x1 && light_position.x < x2
                    || light_position.y > y1 && light_position.y < y2
                {
                    // Light is in the crosshairs of the tile
                    // Sort the points by distance from the light
                    points.sort_by(|a, b| {
                        let d1 = glm::distance(a, &light_position);
                        let d2 = glm::distance(b, &light_position);
                        d1.partial_cmp(&d2).unwrap()
                    });
                } else {
                    // Light is diagonal to the tile
                    batch.rect(
                        &RectF {
                            x: x1,
                            y: y1,
                            w: TILE_SIZE as f32,
                            h: TILE_SIZE as f32,
                        },
                        base_color,
                    );
                    points.sort_by(|a, b| {
                        let d1 = glm::distance(a, &light_position);
                        let d2 = glm::distance(b, &light_position);
                        d1.partial_cmp(&d2).unwrap()
                    });
                    points.rotate_right(1);
                    points[0] = points[2];
                    points[1] = points[3];
                }

                // Take the furtherest two points, and project them outwards from the light
                let distance_from_light = points[2] - light_position;
                let distance_from_light_norm = glm::normalize(&distance_from_light);
                points[2].x += distance_from_light_norm.x * PROJECTION_DISTANCE * 2.0;
                points[2].y += distance_from_light_norm.y * PROJECTION_DISTANCE * 2.0;
                let distance_from_light = points[3] - light_position;
                let distance_from_light_norm = glm::normalize(&distance_from_light);
                points[3].x += distance_from_light_norm.x * PROJECTION_DISTANCE * 2.0;
                points[3].y += distance_from_light_norm.y * PROJECTION_DISTANCE * 2.0;

                let points = points.into_iter().map(|p| (p.x, p.y)).collect::<Vec<_>>();

                batch.circle_fan(
                    (x1 + TILE_SIZE as f32 / 2.0, y1 + TILE_SIZE as f32 / 2.0),
                    &vec![points[0], points[1], points[3], points[2]],
                    light_color,
                );
            }

            // Draw a circle (stencil out the shadows)
            batch.set_stencil(Stencil::mask(0));
            batch.rect(
                &RectF {
                    x: light_position.x - PROJECTION_DISTANCE / 2f32,
                    y: light_position.y - PROJECTION_DISTANCE / 2f32,
                    w: PROJECTION_DISTANCE,
                    h: PROJECTION_DISTANCE,
                },
                light_color,
            );

            batch.set_stencil(Stencil::disable());
            batch.render(&target, ortho);
            batch.clear();
        }

        batch.set_blend(blend::NORMAL);
        batch.clear();
    }
}
