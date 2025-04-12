use std::{rc::Rc};

use crate::{
    content,
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, TILE_SIZE},
};
use engine::{
    ecs::component::Component,
    graphics::{
        batch::Batch,
        common::RectF,
        texture::{SubTexture, Texture},
    },
};
use ldtk_rust::Level;

#[derive(Clone, Copy, PartialEq)]
pub struct Tile {
    pub x: u32, // pixel coordinates in the layer
    pub y: u32,
    pub src_x: i64, // pixel coordinates in the tileset
    pub src_y: i64,
    pub kind: u32,
}

#[derive(PartialEq, Debug)] // Ensure PartialEq is derived for comparison

pub enum LayerType {
    Tiles,
    Entities,
}
pub struct MapEntity {
    pub px: i32,
    pub py: i32,
    pub identifier: String,
    pub custom_fields: Vec<String>,
}
pub struct Layer {
    tileset_id: i64,
    pub kind: LayerType,
    pub tiles: Vec<Tile>,
    pub entities: Vec<MapEntity>,
}

#[allow(dead_code)]
pub struct Room {
    pub world_position: glm::Vec2,
    pub layers: Vec<Layer>,
    pub rect: RectF,
    // This is essentially the camera in world space, move out of here?
    pub camera_ortho: glm::Mat4,
    albedo_texture: Option<SubTexture>,
    normal_texture: Option<SubTexture>,
}
impl Room {
    /**
     * Returs the normal texture for this map, it will render it needed
     */
    pub fn normal(&self) -> SubTexture {
        self.normal_texture
            .as_ref()
            .expect("missing normal, did you forget to call pre-render()?")
            .clone()
    }
    /**
     * Returs the color texture for this map, it will render it needed
     */
    pub fn albedo(&self) -> SubTexture {
        self.albedo_texture
            .as_ref()
            .expect("missing albedo, did you forget to call pre-render()?")
            .clone()
    }

    pub fn from_level(level: &Level) -> Self {
        let mut layers: Vec<Layer> = Vec::new();
        for layer in level.layer_instances.as_ref().unwrap() {
            match layer.layer_instance_type.as_str() {
                "Tiles" => {
                    let tiles = layer
                        .grid_tiles
                        .iter()
                        .map(|f| Tile {
                            x: f.px[0] as u32,
                            y: f.px[1] as u32,
                            src_x: f.src[0],
                            src_y: f.src[1],
                            kind: f.t as u32,
                        })
                        .collect();
                    layers.push(Layer {
                        kind: LayerType::Tiles,
                        tileset_id: layer.tileset_def_uid.expect("Missing tileset id"),
                        tiles,
                        entities: vec![],
                    })
                }
                "Entities" => {
                    let map_entities: Vec<MapEntity> = layer
                        .entity_instances
                        .iter()
                        .map(|entity_instance| {
                            let mut custom_fields: Vec<String> = vec![];
                            for instance in entity_instance.field_instances.iter() {
                                if let Some(v) = instance.value.as_ref() {
                                    if v.is_string() {
                                        let v = v.as_str().unwrap();
                                        custom_fields.push(String::from(v));
                                    }
                                }
                            }
                            return MapEntity {
                                identifier: entity_instance.identifier.clone(),
                                px: entity_instance.px[0] as i32,
                                py: entity_instance.px[1] as i32,
                                custom_fields,
                            };
                        })
                        .collect();
                    layers.push(Layer {
                        tileset_id: layer.layer_def_uid, // do I use this for anything?
                        kind: LayerType::Entities,
                        tiles: vec![],
                        entities: map_entities,
                    })
                }
                _ => {}
            }
        }

        assert!(
            level.px_wid == GAME_PIXEL_WIDTH as i64,
            "Level width must be GAME_PIXEl_WIDTH"
        );
        assert!(
            level.px_hei == GAME_PIXEL_HEIGHT as i64,
            "Level width must be GAME_PIXEL_HEIGHT"
        );

        let rect = RectF {
            x: level.world_x as f32,
            y: level.world_y as f32,
            w: GAME_PIXEL_WIDTH as f32,
            h: GAME_PIXEL_HEIGHT as f32,
        };

        Room {
            world_position: glm::Vec2::new(level.world_x as f32, level.world_y as f32),
            layers,
            rect,
            albedo_texture: None,
            normal_texture: None,
            camera_ortho: glm::ortho(
                level.world_x as f32,
                level.world_x as f32 + GAME_PIXEL_WIDTH as f32,
                level.world_y as f32 + GAME_PIXEL_HEIGHT as f32,
                level.world_y as f32,
                -1.0,
                1.0,
            ),
        }
    }

    pub fn prerender_normals(&mut self, batch: &mut Batch) {
        for layer in self.layers.iter().rev() {
            if let LayerType::Entities = layer.kind {
                continue;
            }
            let tileset = content().tilesets.get(&layer.tileset_id).unwrap();
            for tile in layer.tiles.iter() {
                let tile_rect = RectF {
                    x: tile.x as f32,
                    y: tile.y as f32,
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                };
                batch.sprite(
                    &tile_rect,
                    &SubTexture::new(
                        Rc::clone(&tileset.normal),
                        RectF {
                            x: tile.src_x as f32,
                            y: tile.src_y as f32,
                            w: tileset.tile_size as f32,
                            h: tileset.tile_size as f32,
                        },
                    ),
                    (1f32, 1f32, 1f32, 1f32),
                );
            }
        }
    }

    pub fn prerender_colors(&mut self, batch: &mut Batch) {
        // Render room
        for layer in self.layers.iter().rev() {
            if let LayerType::Entities = layer.kind {
                continue;
            }
            let tileset = content().tilesets.get(&layer.tileset_id).unwrap();
            for tile in layer.tiles.iter() {
                let tile_rect = RectF {
                    x: tile.x as f32,
                    y: tile.y as f32,
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                };
                batch.sprite(
                    &tile_rect,
                    &SubTexture::new(
                        Rc::clone(&tileset.texture),
                        RectF {
                            x: tile.src_x as f32,
                            y: tile.src_y as f32,
                            w: tileset.tile_size as f32,
                            h: tileset.tile_size as f32,
                        },
                    ),
                    (1f32, 1f32, 1f32, 1f32),
                );
            }
        }
    }

    pub fn set_color_texture(&mut self, color: Rc<Texture>) {
        self.albedo_texture = Some(SubTexture::new(color.clone(), self.rect.clone()));
    }
    pub fn set_normal_texture(&mut self, color: Rc<Texture>) {
        self.normal_texture = Some(SubTexture::new(color.clone(), self.rect.clone()));
    }
}
impl Component for Room {
    const CAPACITY: usize = 8;
}
