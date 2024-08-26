use engine::{ecs::WorldOp, graphics::batch::Batch};
use ldtk_rust::Project;

use crate::{
    components::{
        collider::{Collider, ColliderType},
        position::Position,
        room::Room,
    },
    GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE,
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
    fn init(&mut self, world: &mut impl WorldOp) {}
    fn destroy(&mut self, world: &mut impl WorldOp) {}
}

pub struct GameScene {
    pub room_index: u32,
    entities: Vec<u32>,
}
impl GameScene {
    pub fn with_map(index: u32) -> Self {
        GameScene {
            room_index: index,
            entities: Vec::new(),
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self, world: &mut impl WorldOp) {
        let mut room_entity = world.add_entity();

        let ldtk = Project::new("src/map.ldtk");
        let level = ldtk
            .levels
            .get(self.room_index as usize)
            .expect("No level present in ldtk");
        let room = Room::from_level(level);

        room_entity.assign(Position::new(level.world_x as i32, level.world_y as i32));
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
        self.entities.push(room_entity.id);
    }

    fn destroy(&mut self, world: &mut impl WorldOp) {
        for entity in self.entities.drain(..) {
            world.remove_entity(entity);
        }
    }
}
