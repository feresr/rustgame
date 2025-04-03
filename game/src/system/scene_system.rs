use common::Keyboard;
use engine::{
    ecs::{World, WorldOp},
    graphics::batch::Batch,
};
use ldtk_rust::Project;

use crate::{
    components::{player::Player, position::Position, room::Room},
    content, current_room,
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH},
    scene::{GameScene, Scene},
};

/**
 * Determines what Room the player is currently in.
 * Updating the current room if necessary.
 */
pub struct SceneSystem {
    pub initialised: bool,
    pub camera: glm::Mat4,
    pub scene: GameScene,
}
impl SceneSystem {
    pub fn new() -> Self {
        let (x, y) = (0, 1);
        let camera = glm::ortho(
            (x * GAME_PIXEL_WIDTH) as f32,
            (x * GAME_PIXEL_WIDTH + GAME_PIXEL_WIDTH) as f32,
            (y * GAME_PIXEL_HEIGHT) as f32,
            (y * GAME_PIXEL_HEIGHT + GAME_PIXEL_HEIGHT) as f32,
            0.0f32,
            2f32,
        );
        SceneSystem {
            initialised: false,
            camera: camera,
            scene: GameScene::with_room(x as i32, y as i32),
        }
    }

    pub fn update(&mut self, world: &mut World, keyboard: &Keyboard) {
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

            self.camera = glm::ortho(
                (room_x * GAME_PIXEL_WIDTH) as f32,
                (room_x * GAME_PIXEL_WIDTH + GAME_PIXEL_WIDTH) as f32,
                (room_y * GAME_PIXEL_HEIGHT) as f32,
                (room_y * GAME_PIXEL_HEIGHT + GAME_PIXEL_HEIGHT) as f32,
                0.0f32,
                2f32,
            );
        }
    }
}

pub struct Map {
    width: usize,
    height: usize,
    rooms: Vec<Option<Room>>,
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
            dbg!(x);
            dbg!(y);
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
}
