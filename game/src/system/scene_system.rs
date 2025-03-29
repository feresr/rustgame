use engine::ecs::{World, WorldOp};

use crate::{
    components::{player::Player, position::Position}, game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH}, scene::{GameScene, Scene}
};

/**
 * Determines what Room the player is currently in.
 * Updating the current room if necessary.
 */
pub struct SceneSystem {
    pub current_room: (usize, usize),
    pub camera: glm::Mat4,
    pub map: Map,
    pub scene: GameScene,
}
impl SceneSystem {
    pub fn new() -> Self {
        let mut map = Map::new(2, 2);
        map.set(0, 0, 2);
        map.set(0, 1, 0);
        map.set(1, 0, 3);
        map.set(1, 1, 1);
        let ortho = glm::ortho(
            (0 * GAME_PIXEL_WIDTH) as f32,
            (0 * GAME_PIXEL_WIDTH + GAME_PIXEL_WIDTH) as f32,
            (0 * GAME_PIXEL_HEIGHT) as f32,
            (0 * GAME_PIXEL_HEIGHT + GAME_PIXEL_HEIGHT) as f32,
            0.0f32,
            2f32,
        );
        let current_room = (0, 0);
        let first = map.get(current_room.0, current_room.1).unwrap();
        SceneSystem {
            current_room,
            camera: ortho,
            map,
            scene: GameScene::with_map(first),
        }
    }
    pub fn update(&mut self, world: &mut World) {
        let room_x;
        let room_y;
        {
            let player = world.first::<Player>().expect("Player not found");
            let position = player.get::<Position>();
            room_x = (position.x as f32 / GAME_PIXEL_WIDTH as f32) as usize;
            room_y = (position.y as f32 / GAME_PIXEL_HEIGHT as f32) as usize;
        }

        if (room_x, room_y) != self.current_room {
            // We are in a different room.
            self.current_room = (room_x, room_y);
            let new_level = self
                .map
                .get(self.current_room.0, self.current_room.1)
                .unwrap();
            let new_scene = GameScene::with_map(new_level.to_owned());
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
    height: usize,
    width: usize,
    room_names: Vec<Option<u32>>,
}
impl Map {
    fn new(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            room_names: vec![None; width * height],
        }
    }
    fn set(&mut self, x: usize, y: usize, path: u32) {
        assert!(x < self.width);
        assert!(y < self.height);
        self.room_names[x + y * self.width] = Some(path);
    }
    fn get(&self, x: usize, y: usize) -> Option<u32> {
        assert!(x < self.width, "x: {} < w: {}", x, self.width);
        assert!(y < self.height, "y: {} < h: {}", y, self.height);
        if let Some(s) = &self.room_names[x + y * self.width] {
            return Some(s.to_owned());
        }
        return None;
    }
}
