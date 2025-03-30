use crate::{
    components::{
        light::Light,
        position::Position,
        room::{Room, Tile},
    },
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, TILE_SIZE},
};
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
use std::{num::Wrapping, rc::Rc};

// todo a_type is (mult wash fill pad) document better
const FRAGMENT_SHADER_SOURCE: &str = include_str!("light_shader.fs");

pub struct LightSystem {
    target: Target, // Where it's going to render the lights - temporarily
    material: Material,
    time: Wrapping<u32>,
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
            target,
            material,
            time: Wrapping(0),
        }
    }

    pub fn color(&self) -> Rc<Texture> {
        return self.target.attachments[0].clone()
    }

    pub fn render(&mut self, world: &World, batch: &mut Batch) {
        self.time += 1;

        let base_color = (0.0, 0.0, 0.0, 1.0);
        let light_color = (1.00, 1.00, 1.00, 1.0);

        let room_entity = world.all_with::<Room>().next().expect("No Room present");
        let room = room_entity.get::<Room>();
        let room_position = room_entity.get::<Position>();

        let projection_distance: f32 = 140.0 + 5.0f32 * f32::sin(self.time.0 as f32 / 60f32);

        // TODO: this is the camera. should the camera be part of the world (an entity)?
        let ortho = &room.world_ortho;
        self.target.clear(base_color);
        // Make the target non-drawable
        for light_entity in world.all_with::<Light>() {
            batch.push_material(&self.material);
            self.target.clear_stencil(0);
            let light_offset = light_entity.get::<Light>();
            let light_position = light_entity.get::<Position>().as_vec2();
            let light_position =
                light_position.xy() + glm::vec2(light_offset.offset_x, light_offset.offset_y);

            // normalise light position (0 - 1) for the shader
            let ligh_posx = light_position.x - room_position.x as f32;
            let ligh_posy = light_position.y - room_position.y as f32;
            self.material
                .set_value2f("u_light_position", (ligh_posx, ligh_posy));
            self.material
                .set_valuef("u_light_radius", projection_distance / 2f32);

            // Draw oclusion shadows (in the stencil buffer)
            batch.set_stencil(Stencil::write(1));
            batch.set_blend(blend::ADDITIVE);
            if let Some(layer) = room.layers.first() {
                for tile in layer.tiles.iter() {
                    self.draw_shadow(
                        batch,
                        light_position,
                        projection_distance,
                        room_position.to_owned(),
                        tile,
                    );
                }
            }

            // Draw a circle (stencil out the shadows)
            batch.set_stencil(Stencil::mask(0));
            batch.rect(
                &RectF {
                    x: light_position.x - projection_distance / 2f32,
                    y: light_position.y - projection_distance / 2f32,
                    w: projection_distance,
                    h: projection_distance,
                },
                light_color,
            );

            batch.set_stencil(Stencil::disable());
            batch.render(&self.target, ortho);
            batch.clear();
        }
        // batch.pop_material();

        batch.set_blend(blend::NORMAL);
        batch.clear();
    }

    fn draw_shadow(
        &self,
        batch: &mut Batch,
        light_position: glm::Vec2,
        projection_distance: f32,
        room_position: Position,
        tile: &Tile,
    ) {
        let base_color = (0.0, 0.0, 0.0, 1.0);
        let light_color = (1.00, 1.00, 1.00, 1.0);

        let tile_position = glm::vec2(
            room_position.x as f32 + tile.x as f32,
            room_position.y as f32 + tile.y as f32,
        );

        let tile_light_distance = glm::distance(&tile_position, &light_position);
        if tile_light_distance > projection_distance {
            return;
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
        points[2].x += distance_from_light_norm.x * projection_distance * 2.0;
        points[2].y += distance_from_light_norm.y * projection_distance * 2.0;
        let distance_from_light = points[3] - light_position;
        let distance_from_light_norm = glm::normalize(&distance_from_light);
        points[3].x += distance_from_light_norm.x * projection_distance * 2.0;
        points[3].y += distance_from_light_norm.y * projection_distance * 2.0;

        let points = points.into_iter().map(|p| (p.x, p.y)).collect::<Vec<_>>();

        // Using the center of the tile as the center, fan triangles to draw the shadow
        batch.circle_fan(
            (x1 + TILE_SIZE as f32 / 2.0, y1 + TILE_SIZE as f32 / 2.0),
            &vec![points[0], points[1], points[3], points[2]],
            light_color,
        );
    }
}
