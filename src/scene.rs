use engine::ecs::WorldOp;

use crate::{
    components::{
        collider::{Collider, ColliderType},
        light::Light,
        position::Position,
        room::Room,
        sprite::Sprite,
    },
    content, GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE,
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
    fn init(&mut self, _world: &mut impl WorldOp) {}
    fn destroy(&mut self, _world: &mut impl WorldOp) {}
}

pub struct GameScene {
    pub room_index: u32,
    entities: Vec<u32>,
    pub camera : glm::Mat4
}
impl GameScene {
    pub fn with_map(index: u32) -> Self {
        GameScene {
            room_index: index,
            entities: Vec::new(),
            camera: glm::Mat4::identity()
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self, world: &mut impl WorldOp) {
        let mut room_entity = world.add_entity();
        let ldtk = &content().ldkt;
        let level = ldtk
            .levels
            .get(self.room_index as usize)
            .expect("No level present in ldtk");

        let room = Room::from_level(level);
        self.camera = room.world_ortho;

        room_entity.assign(Position::new(level.world_x as i32, level.world_y as i32));
        let mut collisions = vec![false; GAME_TILE_WIDTH * GAME_TILE_HEIGHT];
        for tile in room.layers.first().unwrap().tiles.iter() {
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

        // Entities
        for layer in level.layer_instances.as_ref().unwrap() {
            match layer.layer_instance_type.as_str() {
                "Entities" => {
                    for entity in layer.entity_instances.iter() {
                        let mut e = world.add_entity();
                        e.assign(Position {
                            x: level.world_x as i32 + entity.px[0] as i32,
                            y: level.world_y as i32 + entity.px[1] as i32,
                        });
                        for field in entity.field_instances.iter() {
                            let f = field.value.as_ref().unwrap();
                            e.assign(Sprite::new(&content().sprites[f.as_str().unwrap()]));
                            e.assign(Light {})
                        }
                        self.entities.push(e.id);
                    }
                }
                _ => {}
            }
        }
    }

    fn destroy(&mut self, world: &mut impl WorldOp) {
        for entity in self.entities.drain(..) {
            world.remove_entity(entity);
        }
    }
}
