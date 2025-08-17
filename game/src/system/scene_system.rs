use crate::{
    components::{player::Player, position::Position},
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH},
    scene::{GameScene, Scene},
};
use common::Debug;
use engine::ecs::{World, WorldOp};

pub const OUTLINE_SHADER: &str = include_str!("outline.fs");

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
            scene: GameScene::with_room(x, y),
        }
    }

    pub fn update(&mut self, world: &mut World) {
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

        // Debug::display(&format!("Player in room: {} {} ", room_x, room_y));
        if (room_x, room_y) != (self.scene.room_x as usize, self.scene.room_y as usize) {
            let new_scene = GameScene::with_room(room_x as i32, room_y as i32);
            self.scene.destroy(world);
            self.scene = new_scene;
            self.scene.init(world);
        }
    }
}
