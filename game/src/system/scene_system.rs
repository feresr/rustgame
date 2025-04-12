use common::Keyboard;
use engine::{
    ecs::{World, WorldOp},
    graphics::{batch::Batch, target::Target},
};
use ldtk_rust::Project;

use crate::{
    components::{player::Player, position::Position, room::Room},
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH},
    scene::{GameScene, Scene},
};

/**
 * Determines what Room the player is currently in.
 * Updating the current room if necessary.
 */
pub struct SceneSystem {
    pub initialised: bool,
    pub scene: GameScene,
}
impl SceneSystem {
    pub fn new() -> Self {
        let (x, y) = (0, 0);
        SceneSystem {
            initialised: false,
            scene: GameScene::with_room(x as i32, y as i32),
        }
    }

    pub fn update(&mut self, world: &mut World, _: &Keyboard) {
        if !self.initialised {
            // TODO remove this
            self.scene.init(world);
            self.initialised = true
        }

        let room_x;
        let room_y;
        {
            let player = world.first::<Player>().expect("Player not found");
            let position = player.get::<Position>();
            room_x = (position.x as f32 / GAME_PIXEL_WIDTH as f32) as usize;
            room_y = (position.y as f32 / GAME_PIXEL_HEIGHT as f32) as usize;
        }

        if (room_x, room_y) != (self.scene.room_x as usize, self.scene.room_y as usize) {
            let new_scene = GameScene::with_room(room_x as i32, room_y as i32);
            self.scene.destroy(world);
            self.scene = new_scene;
            self.scene.init(world);
        }
    }
}

pub struct Map {
    width: usize,
    height: usize,
    pub rooms: Vec<Option<Room>>,
}
impl Map {
    pub fn new(ldtk: &Project) -> Self {
        let map_width = 2; // ldtk.world_grid_width.unwrap() as usize;
        let map_height = 2; // ldtk.world_grid_height.unwrap() as usize;
        let room_count = map_width * map_height;

        let mut rooms = Vec::with_capacity(room_count);
        rooms.resize_with(room_count, || None);

        for level in ldtk.levels.iter() {
            let room = Room::from_level(level);
            let x = level.world_x / GAME_PIXEL_WIDTH as i64;
            let y = level.world_y / GAME_PIXEL_HEIGHT as i64;
            let index = (x + (y * map_width as i64)) as usize;
            rooms[index] = Some(room);
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
        self.rooms[x + (y * self.width)]
            .as_mut()
            .expect("Missing room")
    }

    pub fn prerender(&mut self, batch: &mut Batch, color_target: &Target, normal_target: &Target) {
        for (_, room) in self.rooms.iter_mut().enumerate() {
            if let Some(room) = room.as_mut() {
                batch.push_matrix(glm::translation(&glm::vec3(
                    room.world_position.x,
                    room.world_position.y,
                    0.0,
                )));
                room.prerender_colors(batch);
                batch.pop_matrix();

                room.set_color_texture(color_target.color());
            }
            batch.simple_render(&color_target);
        }

        for (_, room) in self.rooms.iter_mut().enumerate() {
            if let Some(room) = room.as_mut() {
                batch.push_matrix(glm::translation(&glm::vec3(
                    room.world_position.x,
                    room.world_position.y,
                    0.0,
                )));
                room.prerender_normals(batch);
                batch.pop_matrix();
                room.set_normal_texture(normal_target.color());
            }
            batch.simple_render(&normal_target);
        }
    }
}
