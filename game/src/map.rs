use crate::components::room::{Room};
use crate::game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, ROOM_COUNT_H, ROOM_COUNT_W};
use crate::system::scene_system::OUTLINE_SHADER;
use engine::graphics;
use engine::graphics::batch::Batch;
use engine::graphics::common::RectF;
use engine::graphics::material::Material;
use engine::graphics::target::Target;
use engine::graphics::texture::TextureFormat;
use ldtk_rust::Project;
use serde::{Deserialize, Serialize};

pub struct Map {
    width: usize,
    height: usize,
    pub rooms: Vec<Room>,
}

impl Map {
    pub fn empty() -> Self {
        Self {
            width: ROOM_COUNT_W,
            height: ROOM_COUNT_H,
            rooms: (0..4)
                .map(move |y| (0..4).map(move |x| Room::empty((x as u32, y as u32))))
                .flatten()
                .collect(),
        }
    }

    pub fn new(ldtk: &Project) -> Self {
        let map_width = 5; // ldtk.world_grid_width.unwrap() as usize;
        let map_height = 2; // ldtk.world_grid_height.unwrap() as usize;
        let room_count = map_width * map_height;

        let mut rooms = Vec::with_capacity(room_count);

        for level in ldtk.levels.iter() {
            let room = Room::from_level(level);
            let x = level.world_x / GAME_PIXEL_WIDTH as i64;
            let y = level.world_y / GAME_PIXEL_HEIGHT as i64;
            let index = (x + (y * map_width as i64)) as usize;
            rooms.push(room);
        }
        Map {
            width: map_width,
            height: map_height,
            rooms,
        }
    }

    pub fn get(&mut self, x: usize, y: usize) -> &mut Room {
        assert!(x < self.width, "x: {} < w: {}", x, self.width);
        assert!(y < self.height, "y: {} < h: {}", y, self.height);
        &mut self.rooms[x + (y * self.width)]
    }

    pub fn prerender(
        &mut self,
        batch: &mut Batch,
        color_target: &Target,
        normal_target: &Target,
        outline_target: &Target,
    ) {
        color_target.clear((0f32, 0f32, 0f32, 0f32));
        normal_target.clear((0f32, 0f32, 0f32, 0f32));
        outline_target.clear((0f32, 0f32, 0f32, 0f32));

        for room in self.rooms.iter_mut(){
            batch.push_matrix(glm::translation(&glm::vec3(
                room.world_position.x,
                room.world_position.y,
                0.0,
            )));
            room.prerender_colors(batch);
            batch.pop_matrix();

            room.set_color_texture(color_target.color());
            batch.render(&color_target);
        }

        for room in self.rooms.iter_mut() {
            batch.push_matrix(glm::translation(&glm::vec3(
                room.world_position.x,
                room.world_position.y,
                0.0,
            )));
            room.prerender_normals(batch);
            batch.pop_matrix();
            room.set_normal_texture(normal_target.color());
            batch.render(&normal_target);
        }

        // Write outlined normals

        batch.clear();

        // Write solid block in to this temp target
        let temp = Target::new(
            color_target.width,
            color_target.height,
            &[TextureFormat::RGBA],
        );
        for (_, room) in self.rooms.iter_mut().enumerate() {
            batch.push_matrix(glm::translation(&glm::vec3(
                room.world_position.x,
                room.world_position.y,
                0.0,
            )));
            room.prerender_outlines(batch);
            batch.pop_matrix();
            room.set_outline_texture(outline_target.color());
            batch.render(&temp);
        }
        batch.clear();
        let outline_shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, OUTLINE_SHADER);
        let material = Material::new(outline_shader);
        material.set_vector2f(
            "u_texelSize",
            &[
                1.0f32 / color_target.width as f32,
                1.0f32 / color_target.height as f32,
            ],
        );
        batch.push_material(&material);
        let rect = RectF::with_size(color_target.width as f32, color_target.height as f32);
        batch.tex(&rect, temp.color(), (1f32, 1f32, 1f32, 1f32));
        batch.render(outline_target);
        batch.pop_material();
        batch.clear();
    }
}
