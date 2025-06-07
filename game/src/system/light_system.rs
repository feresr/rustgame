use crate::{
    components::{
        light::Light,
        position::Position,
        room::{LayerType, Tile},
    },
        game_state::{GameState, TILE_SIZE},
};
use engine::{
    ecs::{World, WorldOp},
    graphics::{
        self,
        batch::{Batch, Stencil},
        blend::{self, ADDITIVE},
        common::{EdgeF, RectF},
        material::Material,
        target::Target,
        texture::TextureSampler,
    },
};
use glm::{not, proj};
use std::num::Wrapping;

pub const NORMAL_OUTLINE_FRAGMENT_SOURCE: &str = include_str!("light_normals_outline.fs");

pub struct LightSystem {
    material: Material,
    normal_material: Material,
    time: Wrapping<u32>,
}

impl LightSystem {
    pub fn new() -> Self {
        // TODO: could this be TextureFormat:R?
        let shader = graphics::shader::Shader::new(
            graphics::VERTEX_SHADER_SOURCE,
            graphics::FRAGMENT_SHADER_SOURCE,
        );
        let normal_shader = graphics::shader::Shader::new(
            graphics::VERTEX_SHADER_SOURCE,
            NORMAL_OUTLINE_FRAGMENT_SOURCE,
        );
        let material = Material::with_sampler(shader, TextureSampler::nearest());
        let normal_material = Material::with_sampler(normal_shader, TextureSampler::nearest());

        LightSystem {
            material,
            normal_material,
            time: Wrapping(0),
        }
    }

    pub fn render(&mut self, world: &World, batch: &mut Batch, target: &mut Target) {
        self.time += 1;

        let base_color = (0.0, 0.0, 0.0, 1.0);

        let room = GameState::current_room();
        let room_position = Position::new(room.world_position.x as i32, room.world_position.y as i32);

        let projection_distance: f32 = 140.0 + 5.0f32 * f32::sin(self.time.0 as f32 / 60f32);

        // TODO: this is the camera. should the camera be part of the world (an entity)?
        let ortho = &room.camera_ortho;

        target.clear(base_color);
        // Make the target non-drawable
        for light_entity in world.all_with::<Light>() {
            batch.push_material(&self.material);
            target.clear_stencil(0);
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
            target.clear_stencil(0);
            // batch.set_blend(ADDITIVE);
            batch.set_stencil(Stencil::increment());

            for layer in room
                .layers
                .iter()
                .filter(|l| matches!(&l.kind, LayerType::Tiles(_)))
            {
                for (x, y, tile) in layer.tiles() {
                    let tile_position = glm::vec2(
                        room_position.x as f32 + x as f32 * TILE_SIZE as f32,
                        room_position.y as f32 + y as f32 * TILE_SIZE as f32,
                    );
                    // draw_shadow, only if tile is close enough to the light
                    if glm::distance(&tile_position, &light_position) < projection_distance {
                        self.draw_shadow(
                            tile_position,
                            batch,
                            light_position,
                            projection_distance,
                            room_position.to_owned(),
                            tile,
                        );
                    }
                }
            }

            {
                // Draw all outlines
                batch.set_stencil(Stencil::decr());
                self.normal_material
                    .set_value2f("u_lightPosition", (ligh_posx, ligh_posy));

                batch.push_material(&self.normal_material);
                // Draw outlines here
                batch.sprite(
                    &room.rect,
                    &room.outline(),
                    (1.0f32, 1.0f32, 1.0f32, 1.0f32),
                );
                batch.set_stencil(Stencil::disable());
                batch.pop_material();

                // Remove outlines pointing away from the light
                //
            }

            // Draw a circle (stencil set up above wont' let us paint where shadows are — which is what we want)
            batch.set_stencil(Stencil::mask_eq(0));

            batch.circle(
                (light_position.x, light_position.y),
                92f32,
                38,
                (1f32, 1f32, 1f32, 1f32),
            );

            batch.render_with_projection(target, &ortho);
            target.clear_stencil(0);
            batch.clear();
        }
        // batch.pop_material();

        batch.set_blend(blend::NORMAL);
        batch.clear();
    }

    fn draw_shadow(
        &self,
        tile_position : glm::Vec2,
        batch: &mut Batch,
        light_position: glm::Vec2,
        projection_distance: f32,
        room_position: Position,
        tile: &Tile,
    ) {
        let base_color = (0.0, 0.0, 0.0, 1.0);
        let light_color = (1.00, 1.00, 1.00, 1.0);

        // let tile_position = glm::vec2(
        //     room_position.x as f32 + tile.x as f32,
        //     room_position.y as f32 + tile.y as f32,
        // );

        let tile_light_distance = glm::distance(&tile_position, &light_position);
        if tile_light_distance > projection_distance {
            return;
        }

        let padding = 0f32;
        let _x1 = tile_position.x + padding;
        let _y1 = tile_position.y + padding;
        // let x2 = x1 + TILE_SIZE as f32 - (padding * 2f32);
        // let y2 = y1 + TILE_SIZE as f32 - (padding * 2f32);

        let rect = RectF {
            x: tile_position.x,
            y: tile_position.y,
            w: TILE_SIZE as f32,
            h: TILE_SIZE as f32,
        };

        let center = rect.center();
        let center_vec: glm::Vec2 = glm::Vec2::from(&center);
        let light = glm::normalize(&(light_position - &center_vec));
        let mut edges = rect.edges();
        let mut facing_light: Vec<&mut EdgeF> = edges
            .iter_mut()
            .filter(|edge| {
                let dot = glm::normalize_dot(&light, &edge.normal());
                dot < 0.0f32
            })
            .collect();

        // rect.scale(0.5f32);
        batch.rect(&rect, base_color);

        // Only 1 or 2 edges face the light at any given time
        assert!(facing_light.iter().count() <= 2);

        for edge in facing_light.iter_mut() {
            // let normal = glm::normalize(&edge.normal());
            // edge.translate(&(-normal * 5.0f32));
            let a = edge.a;
            let b = edge.b;

            // Project c and d
            let mut c = edge.a;
            let point_vec = glm::Vec2::from(&c);
            let distance_from_light = glm::normalize(&(point_vec - light_position));
            c.x += distance_from_light.x * projection_distance * 2.0;
            c.y += distance_from_light.y * projection_distance * 2.0;

            let mut d = edge.b;
            let point_vec = glm::Vec2::from(&d);
            let distance_from_light = glm::normalize(&(point_vec - light_position));
            d.x += distance_from_light.x * projection_distance * 2.0;
            d.y += distance_from_light.y * projection_distance * 2.0;

            let avgx = [a, b, c, d].map(|f| f.x).iter().sum::<f32>() / 4f32;
            let avgy = [a, b, c, d].map(|f| f.y).iter().sum::<f32>() / 4f32;

            let all_points = [a, b, d, c].iter().copied().map(Into::into).collect();
            batch.circle_fan((avgx, avgy), &all_points, light_color);
        }

        // points[2].x += distance_from_light_norm.x * projection_distance * 2.0;
        // points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

        // for point in points.iter() {
        //     batch.circle((point.x, point.y), 12f32, 18, (1f32, 1f32, 0f32, 1f32));
        //
        // let padding = 0f32;
        // batch.rect(
        //     &RectF {
        //         x: x1 + padding,
        //         y: y1 + padding,
        //         w: (x2 - x1) as f32 - (2f32 * padding),
        //         h: (y2 - y1) as f32 - (2f32 * padding),
        //     },
        //     base_color,
        // );

        // if light_position.x > x1 && light_position.x < x2
        //     || light_position.y > y1 && light_position.y < y2
        // {
        //     // Light is in the crosshairs of the tile
        //     // Sort the points by distance from the light
        //     points.sort_by(|a, b| {
        //         let d1 = glm::distance(a, &light_position);
        //         let d2 = glm::distance(b, &light_position);
        //         d1.partial_cmp(&d2).unwrap()
        //     });
        // } else {
        //     // Light is diagonal to the tile
        //     points.sort_by(|a, b| {
        //         let d1 = glm::distance(a, &light_position);
        //         let d2 = glm::distance(b, &light_position);
        //         d1.partial_cmp(&d2).unwrap()
        //     });
        //     points.rotate_right(1);
        //     points[0] = points[2];
        //     points[1] = points[3];
        // }

        // // Points are sorded now.
        // // Take the furtherest two points, and project them outwards from the light
        // let distance_from_light = points[2] - light_position;
        // let distance_from_light_norm = glm::normalize(&distance_from_light);
        // points[2].x += distance_from_light_norm.x * projection_distance * 2.0;
        // points[2].y += distance_from_light_norm.y * projection_distance * 2.0;
        // let distance_from_light = points[3] - light_position;
        // let distance_from_light_norm = glm::normalize(&distance_from_light);
        // points[3].x += distance_from_light_norm.x * projection_distance * 2.0;
        // points[3].y += distance_from_light_norm.y * projection_distance * 2.0;

        // let points = points.into_iter().map(|p| (p.x, p.y)).collect::<Vec<_>>();

        // Using the center of the tile as the center, fan triangles to draw the shadow
        // batch.circle_fan(
        //     (x1 + TILE_SIZE as f32 / 2.0, y1 + TILE_SIZE as f32 / 2.0),
        //     &vec![points[0], points[1], points[3], points[2]],
        //     light_color,
        // );
    }
}
