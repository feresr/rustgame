use engine::{ecs::WorldOp, graphics::common::RectF};

use crate::{
    components::{
        button::Button,
        collider::{Collider, ColliderType},
        light::{Light, LightSwitch},
        position::Position,
        room::LayerType,
        sprite::Sprite,
    },
    content::{self, Content}, current_room,
    game_state::{GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE},
};

/**
 * A scene can represent a Pause screen, or a Room in the game, or even a full screen UI Overlay.
 * Scenes can be stacked (PauseScene on top of a PlayScene)
 *
 * All scenes share a common World — Scenes keep track of their own entities ids so that they can be cleared on cleanup
 */
pub trait Scene {
    fn init(&mut self, _world: &mut impl WorldOp) {}
    fn destroy(&mut self, _world: &mut impl WorldOp) {}
}

pub struct GameScene {
    pub room_x: i32,
    pub room_y: i32,
    entities: Vec<u32>,
}

impl GameScene {
    pub fn with_room(x: i32, y: i32) -> Self {
        GameScene {
            room_x: x,
            room_y: y,
            entities: Vec::new(),
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self, world: &mut impl WorldOp) {
        let mut room_entity = world.add_entity();

        let room = current_room();
        room_entity.assign(Position::new(
            room.world_position.x as i32,
            room.world_position.y as i32,
        ));
        let mut collisions = vec![false; GAME_TILE_WIDTH * GAME_TILE_HEIGHT];

        // Todo: make accessing each layer kind a bit easier
        let tile_layer = room
            .layers
            .iter()
            .find(|layer| matches!(layer.kind, LayerType::Tiles(_)))
            .expect("Map must have at least one tile layer (even if empty)");
        for tile in tile_layer.tiles.iter() {
            let x = (tile.x as f32 / TILE_SIZE as f32) as u32;
            let y = (tile.y as f32 / TILE_SIZE as f32) as u32;
            collisions[(x + y * GAME_TILE_WIDTH as u32) as usize] = true;
        }
        // make this a factory to create the room
        room_entity.assign(Collider::new(
            ColliderType::Grid {
                columns: GAME_TILE_WIDTH,
                rows: GAME_TILE_HEIGHT,
                tile_size: TILE_SIZE,
                cells: collisions,
            },
            true,
        ));

        self.entities.push(room_entity.id);

        // Entities
        for layer in room.layers.iter().as_ref() {
            match layer.kind {
                crate::components::room::LayerType::Entities => {
                    for map_entity in layer.entities.iter() {
                        let mut entity = world.add_entity();
                        entity.assign(Position {
                            x: room.world_position.x as i32 + map_entity.px,
                            y: room.world_position.y as i32 + map_entity.py,
                        });
                        entity.assign(Sprite::new(
                            &Content::sprite(map_entity.identifier.as_str()),
                        ));
                        match map_entity.identifier.as_str() {
                            id if id.starts_with("Light") => {
                                entity.assign(Light::new());
                                entity.assign(LightSwitch::new("b3"))
                            }
                            "Button" => {
                                let name = map_entity.custom_fields.first().unwrap().clone();
                                entity.assign(Button {
                                    name: name,
                                    pressed: false,
                                });
                                entity.assign(Collider::new(
                                    ColliderType::Rect {
                                        rect: RectF {
                                            x: -4f32,
                                            y: -5f32,
                                            w: 8f32,
                                            h: 5f32,
                                        },
                                    },
                                    false,
                                ));
                            }
                            _ => {}
                        }
                        self.entities.push(entity.id);
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
