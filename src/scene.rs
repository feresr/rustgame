use engine::{
    ecs::{World, WorldOp},
    graphics::{batch::Batch, common::RectF, texture::Texture},
};

use crate::{
    components::{
        collider::{Collider, ColliderType},
        mover::Mover,
        controller::Controller,
        position::Position,
        room::{Room, Tile},
    },
    Gravity, GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE,
};

/**
 * A scene can represent a Pause screen, or a Room in the game, or even a full screen UI Overlay.
 * Scenes can be stacked (PauseScene on top of a PlayScene)
 *
 * Each scenes has its own world.
 * I've considered a shared world that gets cleared when the Scene needs to be swaped.
 * But some entities need to survive between Scene (Player entity)
 * Moving the player entity from world->world is easier than "clear everything except Player entity"
 * Some scenes have nothing to do with the game (Pause scene)
 */

pub trait Scene {
    fn init(&mut self) {}
    fn update(&mut self) {}
    fn render(&self, batch: &mut Batch);
}

pub struct GameScene {
    pub room_path: String,
    pub world: World,
}
impl GameScene {
    pub fn with_map(path: String) -> Self {
        GameScene {
            room_path: path,
            world: World::new(),
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self) {
        let mut room_entity = self.world.add_entity();
        let room = Room::from_path(&self.room_path.to_owned());
        room_entity.assign(Collider::new(ColliderType::Grid {
            columns: GAME_TILE_WIDTH,
            rows: GAME_TILE_HEIGHT,
            tile_size: TILE_SIZE,
            cells: room.tiles.map(|f| f == Tile::SOLID).to_vec(),
        }));
        room_entity.assign(room);
    }

    fn update(&mut self) {
        self.world.update();
    }

    fn render(&self, batch: &mut Batch) {
        self.world.render(batch);
    }
}
