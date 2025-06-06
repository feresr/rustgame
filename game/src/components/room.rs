extern crate serde_big_array;

#[macro_use]
use serde_big_array::BigArray;
use std::fs;
use std::rc::Rc;

use crate::game_state::{GameState, GAME_TILE_HEIGHT, GAME_TILE_WIDTH};
use crate::{
    content::Content,
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
use rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Solid {
        src_x: i64, // pixel coordinates in the tileset
        src_y: i64,
        kind: u32,
    },
    Empty {},
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum LayerType {
    Tiles(String), // Background and foreground
    Entities,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapEntity {
    pub px: i32,
    pub py: i32,
    pub identifier: String,
    pub custom_fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tiles {
    #[serde(with = "BigArray")]
    tiles: [Tile; GAME_TILE_WIDTH * GAME_TILE_HEIGHT],
}

pub struct TileIterator<'a> {
    tiles: std::slice::Iter<'a, Tile>,
    index: usize,
}

impl<'a> Iterator for TileIterator<'a> {
    // this actually
    type Item = (usize, usize, &'a Tile);

    fn next(&mut self) -> Option<Self::Item> {
        self.tiles.next().map(|tile| {
            let x = self.index % GAME_TILE_WIDTH;
            let y = self.index / GAME_TILE_WIDTH;
            self.index += 1;
            (x, y, tile)
        })
    }
}
impl<'a> IntoIterator for &'a Tiles {
    // into iterator is so that for loops work
    type Item = (usize, usize, &'a Tile);
    type IntoIter = TileIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TileIterator {
            tiles: self.tiles.iter(),
            index: 0,
        }
    }
}

impl Tiles {
    pub fn empty() -> Tiles {
        Tiles {
            tiles: [Tile::Empty {}; GAME_TILE_WIDTH * GAME_TILE_HEIGHT],
        }
    }
    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[y * GAME_TILE_WIDTH + x] = tile
    }
    pub fn get(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[y * GAME_TILE_WIDTH + x]
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        dbg!(self.tiles.len());
        dbg!(y * GAME_TILE_WIDTH + x);
        &mut self.tiles[y * GAME_TILE_WIDTH + x]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Layer {
    tileset_id: i64,
    pub kind: LayerType,
    pub tiles: Tiles,
    pub entities: Vec<MapEntity>,
}

impl Layer {
    pub fn solid_tiles(&self) -> impl Iterator<Item = (usize, usize, &Tile)> {
        self.tiles
            .into_iter()
            .filter(|(x, y, tile)| matches!(tile, Tile::Solid { .. }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapData {
    pub width : u32,
    pub height : u32,
    pub rooms : Vec<SavedRoom>,
}
impl MapData {
    pub fn save(width : u32, height : u32, rooms : &Vec<Room>)  {
        let mut saved_rooms = vec![];
        for  room in rooms.iter() {
            let saved = SavedRoom::from((0,0), room);
            saved_rooms.push(saved);
        }
        
        let map_data = MapData {
            width, height, rooms: saved_rooms,
        };
        let map_data_string = serde_yml::to_string(&map_data).unwrap();
        fs::create_dir_all("rooms/").unwrap();
        fs::write("rooms/world.yml", map_data_string).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedRoom {
    pub world_position: (u32, u32),
    pub layers: Vec<Layer>,
}
impl SavedRoom {
    fn from(world_position: (u32, u32), room: &Room) -> SavedRoom {
        SavedRoom {
            world_position,
            layers: room.layers.clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Room {
    pub world_position: glm::Vec2,
    pub layers: Vec<Layer>,
    pub rect: RectF,
    // This is essentially the camera in world space, move out of here?
    pub camera_ortho: glm::Mat4,
    albedo_texture: Option<SubTexture>,
    normal_texture: Option<SubTexture>,
    outline_texture: Option<SubTexture>,
}
impl Room {
    pub fn save(&self) {
        // let sr = SavedRoom {
        //     world_position: (self.world_position.x as u32, self.world_position.y as u32),
        //     layers: self.layers.clone(),
        // };
        let serialized = serde_yml::to_string(&self.layers).unwrap();
        fs::create_dir_all("rooms/").unwrap();
        println!("Writing to: {}", std::fs::canonicalize("rooms/").unwrap().display());

        fs::write(format!("rooms/foo{}-{}.yml", self.world_position.x, self.world_position.y), serialized).unwrap();
    }
    pub fn from(saved_room: SavedRoom) -> Room {
        let position = saved_room.world_position;
        let rect = RectF {
            x: position.0 as f32 * GAME_PIXEL_WIDTH as f32,
            y: position.1 as f32 * GAME_PIXEL_HEIGHT as f32,
            w: GAME_PIXEL_WIDTH as f32,
            h: GAME_PIXEL_HEIGHT as f32,
        };
        Room {
            world_position: glm::vec2(
                saved_room.world_position.0 as f32,
                saved_room.world_position.1 as f32,
            ),
            layers: saved_room.layers,
            rect,
            camera_ortho: Default::default(),
            albedo_texture: None,
            normal_texture: None,
            outline_texture: None,
        }
    }
    pub fn empty(position: (u32, u32)) -> Room {
        let rect = RectF {
            x: position.0 as f32 * GAME_PIXEL_WIDTH as f32,
            y: position.1 as f32 * GAME_PIXEL_HEIGHT as f32,
            w: GAME_PIXEL_WIDTH as f32,
            h: GAME_PIXEL_HEIGHT as f32,
        };

        Room {
            world_position: glm::Vec2::new(rect.x, rect.y),
            layers: vec![Layer {
                tileset_id: 0,
                kind: LayerType::Tiles("empty".to_string()),
                tiles: Tiles::empty(),
                entities: vec![],
            }],
            rect,
            camera_ortho: Default::default(),
            albedo_texture: None,
            normal_texture: None,
            outline_texture: None,
        }
    }

    pub fn from_level(level: &Level) -> Self {
        let mut layers: Vec<Layer> = Vec::new();
        for layer in level.layer_instances.as_ref().unwrap() {
            match layer.layer_instance_type.as_str() {
                "Tiles" => {
                    // let tiles = layer
                    //     .grid_tiles
                    //     .iter()
                    //     .map(|f| Tile::Solid {
                    //         x: f.px[0] as u32,
                    //         y: f.px[1] as u32,
                    //         src_x: f.src[0],
                    //         src_y: f.src[1],
                    //         kind: f.t as u32,
                    //     })
                    //     .collect();
                    layers.push(Layer {
                        kind: LayerType::Tiles(layer.identifier.to_owned()),
                        tileset_id: layer.tileset_def_uid.expect("Missing tileset id"),
                        tiles: Tiles::empty(),
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
                        tiles: Tiles::empty(),
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
            outline_texture: None,
            camera_ortho: glm::ortho(
                level.world_x as f32,
                level.world_x as f32 + GAME_PIXEL_WIDTH as f32,
                level.world_y as f32,
                level.world_y as f32 + GAME_PIXEL_HEIGHT as f32,
                -1.0,
                1.0,
            ),
        }
    }
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

    pub fn outline(&self) -> SubTexture {
        self.outline_texture
            .as_ref()
            .expect("missing outline, did you forget to call pre-render()?")
            .clone()
    }

    pub fn prerender_normals(&mut self, batch: &mut Batch) {
        for layer in self.layers.iter().rev() {
            if let LayerType::Entities = layer.kind {
                continue;
            }
            let tileset = Content::get().tilesets.get(&layer.tileset_id).unwrap();
            for (x, y, tile) in layer.solid_tiles() {
                match tile {
                    Tile::Solid { src_x, src_y, kind } => {
                        let tile_rect = RectF {
                            x: x as f32,
                            y: y as f32,
                            w: TILE_SIZE as f32,
                            h: TILE_SIZE as f32,
                        };
                        batch.sprite(
                            &tile_rect,
                            &SubTexture::new(
                                Rc::clone(&tileset.normal),
                                RectF {
                                    x: *src_x as f32,
                                    y: *src_y as f32,
                                    w: tileset.tile_size as f32,
                                    h: tileset.tile_size as f32,
                                },
                            ),
                            (1f32, 1f32, 1f32, 1f32),
                        );
                    }
                    Tile::Empty { .. } => {}
                }
            }
        }
    }

    pub fn prerender_colors(&mut self, batch: &mut Batch) {
        // Render room
        for layer in self.layers.iter().rev() {
            if let LayerType::Entities = layer.kind {
                continue;
            }
            let tileset = Content::get()
                .tilesets
                .get(&layer.tileset_id)
                .expect("No tileset found");
            for (x, y, tile) in layer.solid_tiles() {
                let tile_rect = RectF {
                    x: x as f32,
                    y: y as f32,
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                };
                match tile {
                    Tile::Solid { src_y, src_x, .. } => {
                        // batch.rect(&tile_rect, (1f32, 1f32, 1f32, 1f32));
                        let r = [0f32, 8f32, 16f32];
                        let mut rand = thread_rng();
                        let random_value = r.choose(&mut rand).unwrap();
                        let random_value2 = r.choose(&mut rand).unwrap();

                        batch.sprite(
                            &tile_rect,
                            &SubTexture::new(
                                Rc::clone(&tileset.texture),
                                RectF {
                                    x: *random_value,
                                    y: *random_value2,
                                    w: 8f32,
                                    h: 8f32,
                                },
                            ),
                            (1f32, 1f32, 1f32, 1f32),
                        );
                    }
                    Tile::Empty { .. } => {}
                }
            }
        }
    }

    pub fn prerender_outlines(&mut self, batch: &mut Batch) {
        return;
        // Render room
        // for layer in self.layers.iter().rev() {
        //     if let LayerType::Tiles(kind) = &layer.kind {
        //         if kind == "Solid" {
        //             let tileset = Content::get().tilesets.get(&layer.tileset_id).unwrap();
        //             for tile in layer.tiles.iter() {
        //                 let tile_rect = RectF {
        //                     x: tile.x as f32,
        //                     y: tile.y as f32,
        //                     w: TILE_SIZE as f32,
        //                     h: TILE_SIZE as f32,
        //                 };
        //                 batch.sprite(
        //                     &tile_rect,
        //                     &SubTexture::new(
        //                         Rc::clone(&tileset.normal),
        //                         RectF {
        //                             x: tile.src_x as f32,
        //                             y: tile.src_y as f32,
        //                             w: tileset.tile_size as f32,
        //                             h: tileset.tile_size as f32,
        //                         },
        //                     ),
        //                     (1f32, 1f32, 1f32, 1f32),
        //                 );
        //             }
        //         }
        //     }
        // }
    }

    pub fn set_color_texture(&mut self, color: Rc<Texture>) {
        self.albedo_texture = Some(SubTexture::new(color.clone(), self.rect.clone()));
    }
    pub fn set_outline_texture(&mut self, color: Rc<Texture>) {
        self.outline_texture = Some(SubTexture::new(color.clone(), self.rect.clone()));
    }
    pub fn set_normal_texture(&mut self, color: Rc<Texture>) {
        self.normal_texture = Some(SubTexture::new(color.clone(), self.rect.clone()));
    }
}
impl Component for Room {
    const CAPACITY: usize = 8;
}
