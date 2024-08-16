use engine::{ecs::World, graphics::batch::Batch};

use crate::components::room::Room;

/**
 * A scene can represent a Pause screen, or a Room in the game, or even a full screen UI Overlay.
 * Scenes can be stacked (PauseScene on top of a PlayScene)
 */

pub trait Scene {
    fn init(&mut self) {}
    fn update(&mut self) {}
    fn render(&self, batch: &mut Batch);
    fn get_player(&self);
}

pub struct GameScene {
    room: Room,
}
impl GameScene {
    pub fn with_map(path: &str) -> Self {
        GameScene {
            room: Room::from_path(path),
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self) {}

    fn update(&mut self) {}

    fn render(&self, batch: &mut Batch) {}

    fn get_player(&self) {}
}
