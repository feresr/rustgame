use engine::{
    ecs::{World, WorldOp},
    graphics::{batch::Batch, common::RectF, texture::Texture},
};

use crate::{
    components::{
        collider::{Collider, ColliderType},
        controller::Controller,
        mover::Mover,
        position::Position,
        room::{Room, Tile},
    },
    content::{self, Content},
    Gravity, GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE,
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
    pub room_index: u32,
    pub world: World,
}
impl GameScene {
    pub fn with_map(index: u32) -> Self {
        GameScene {
            room_index: index,
            world: World::new(),
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self) {
        let mut room_entity = self.world.add_entity();
        let room = Room::from_index(self.room_index);

        let mut collisions = vec![false; GAME_TILE_WIDTH * GAME_TILE_HEIGHT];
        for tile in room.tiles.iter() {
            let x = (tile.x as f32 / TILE_SIZE as f32) as u32;
            let y = (tile.y as f32 / TILE_SIZE as f32) as u32;
            collisions[(x + y * GAME_TILE_WIDTH as u32) as usize] = true;
        }
        // make this a factory to create the room
        room_entity.assign(Collider::new(ColliderType::Grid {
            columns: GAME_TILE_WIDTH,
            rows: GAME_TILE_HEIGHT,
            tile_size: TILE_SIZE,
            cells: collisions,
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
