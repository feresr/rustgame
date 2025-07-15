use std::ops::Not;

use crate::{
    components::room::LayerType,
    game_state::{self, GameState},
};

use common::{Debug, Keyboard, Mouse};
use engine::{
    graphics::{
        batch::Batch,
        common::{PointF, RectF},
    },
    Keycode,
};
use rand::prelude::*;

use crate::components::room::{MapData, Tile};
use crate::game_state::{GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE};
use crate::{
    components::room::Room,
    content::Content,
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH},
    target_manager::TargetManager,
};

pub struct Editor {
    hover_tile: Option<RectF>,
    debug_textures: bool,
    zoom: f32,
    offset: (f32, f32),
}

static mut draw_background_tiles: bool = false;

impl Default for Editor {
    fn default() -> Self {
        Self {
            hover_tile: None,
            debug_textures: false,
            zoom: 1f32,
            offset: (0f32, 0f32),
        }
    }
}

impl Editor {
    pub fn update(&mut self) {
        if Keyboard::pressed(Keycode::Space) {
            self.debug_textures = !self.debug_textures;
        }

        let mouse_pos = Mouse::position();
        let world_mouse = self.screen_to_world((mouse_pos.0, mouse_pos.1));
        let mouse_rel = Mouse::position_rel();
        let world_mouse_rel = (
            mouse_rel.0 as f32 / self.zoom,
            mouse_rel.1 as f32 / self.zoom,
        );

        Debug::window_size("Map editor", 340f32, 200f32);
        Debug::display(&format!(
            "Room count {}",
            Content::map().rooms.len().to_string()
        ));
        Debug::separator();
        Debug::display(&format!("Zoom level: {:.1}", self.zoom));
        Debug::display(&format!(
            "Pan: x:{:.1} y:{:.2}",
            self.offset.0, self.offset.1
        ));
        Debug::separator();
        Debug::display(&format!(
            "Screen mouse: ({:.1},{:.1})",
            mouse_pos.0, mouse_pos.1
        ));
        Debug::display(&format!(
            "World mouse: ({:.1},{:.1})",
            world_mouse.0, world_mouse.1
        ));
        let mut tileset = Content::get().tilesets.get_mut(&0).unwrap();
        let tile_size = tileset.tile_size;
        let texdture_id = tileset.texture.id as usize;
        let mut rng = rand::rng();

        for i in 0..tileset.rows {
            for j in 0..tileset.columns {
                let uv = (
                    (i as u32 * tile_size) as f32 / tileset.texture.width as f32,
                    (j as u32 * tile_size) as f32 / tileset.texture.height as f32,
                );
                let random = (i * 4 + j) as u32; // => unique = j

                if Debug::sprite(
                    random,
                    texdture_id,
                    (tile_size as f32 * 4f32, tile_size as f32 * 4f32),
                    (
                        [uv.0, uv.1],
                        [
                            uv.0 + (tile_size as f32 / tileset.texture.width as f32),
                            uv.1 + (tile_size as f32 / tileset.texture.height as f32),
                        ],
                    ),
                ) {
                    println!("Clikced {}", random);
                };
                Debug::same_line();
            }
            Debug::new_line();
        }

        unsafe {
            Debug::checkbox(
                "Draw background tiles",
                draw_background_tiles,
                Box::new(|| {
                    draw_background_tiles = draw_background_tiles.not();
                }),
            );
        }

        if Keyboard::pressed(Keycode::S) {
            let rooms = &Content::map().rooms;
            MapData::save(4, 4, rooms);
        }

        let room = Content::map().rooms.iter_mut().find(|room| {
            room.rect
                .contains(&PointF::new(world_mouse.0 as f32, world_mouse.1 as f32))
        });
        if room.is_some() {
            let room = room.unwrap();

            // let id = room.albedo().texture.id;
            // Debug::image(id as usize);

            let room_mouse = Self::world_to_room(&room, (world_mouse.0, world_mouse.1));
            Debug::display(&format!(
                "Room mouse: ({:.1},{:.1})",
                room_mouse.0, room_mouse.1
            ));
            Debug::separator();
            Debug::display(&format!(
                "Current room tile count {}",
                room.layers.first().unwrap().tiles().count()
            ));
            Debug::display(&format!(
                "Current room world position: ({:.1},{:.1})",
                room.world_position[0], room.world_position[1]
            ));

            let selected_tile_x: usize =
                ((room_mouse.0 as f32 / GAME_PIXEL_WIDTH as f32) * GAME_TILE_WIDTH as f32) as usize;
            let selected_tile_y: usize = ((room_mouse.1 as f32 / GAME_PIXEL_HEIGHT as f32)
                * GAME_TILE_HEIGHT as f32) as usize;

            Debug::display(&format!(
                "Selected tile: ({:.1},{:.1})",
                selected_tile_x, selected_tile_y
            ));

            let hover_world_pos = Self::room_to_world(
                &room,
                (
                    (selected_tile_x * TILE_SIZE) as i32,
                    (selected_tile_y * TILE_SIZE) as i32,
                ),
            );
            let hover_screen_pos = self.world_to_screen(hover_world_pos);
            let tile_size = TILE_SIZE as f32 * self.zoom;
            self.hover_tile = Some(RectF {
                x: hover_screen_pos.0 as f32,
                y: hover_screen_pos.1 as f32,
                w: tile_size,
                h: tile_size,
            });
            if Mouse::left_held() {
                println!("left_held");
                room.is_dirty = true;
                unsafe {
                    let layer = if draw_background_tiles {
                        room.layers.iter_mut().find(|layer| {
                            matches!(
                                layer.kind,
                                LayerType::Tiles(
                                    crate::components::room::TileLayerType::Background
                                )
                            )
                        })
                    } else {
                        room.layers.iter_mut().find(|layer| {
                            matches!(
                                layer.kind,
                                LayerType::Tiles(
                                    crate::components::room::TileLayerType::Foreground
                                )
                            )
                        })
                    }
                    .unwrap();
                    let kind = layer.tiles.get(selected_tile_x, selected_tile_y).kind;
                    let tile = Tile {
                        src_x: 0,
                        src_y: 0,
                        kind: crate::components::room::TileType::Solid,
                    };
                    layer.tiles.set(selected_tile_x, selected_tile_y, tile);
                }
            }
        }

        // Zoom: https://www.youtube.com/watch?v=ZQ8qtAizis4
        if Mouse::wheel().1 != 0 {
            let screen_position = Mouse::position();
            let before_zoom_world_position = self.screen_to_world(screen_position);
            self.zoom += Mouse::wheel().1 as f32 * 0.1;
            self.zoom = self.zoom.clamp(0.25, 4.0);
            let after_zoom_world_position = self.screen_to_world(screen_position);

            self.offset.0 +=
                after_zoom_world_position.0 as f32 - before_zoom_world_position.0 as f32;
            self.offset.1 +=
                after_zoom_world_position.1 as f32 - before_zoom_world_position.1 as f32;
        }
        if Mouse::right_held() {
            self.offset.0 += world_mouse_rel.0;
            self.offset.1 += world_mouse_rel.1;
        }
    }

    pub fn render(&mut self, batch: &mut Batch, target_manager: &TargetManager) {
        batch.clear();

        if self.debug_textures {
            let maps = target_manager.maps_color.attachments[0].clone();
            let rect = RectF::with_size(maps.width as f32, maps.height as f32);
            batch.tex(&rect, maps, (1f32, 1f32, 1f32, 1f32));

            let maps = target_manager.lights.attachments[0].clone();
            let rect = RectF::with_size(maps.width as f32, maps.height as f32);
            batch.tex(
                &(rect + PointF::new(GAME_PIXEL_WIDTH as f32 * 2f32, 0f32)),
                maps,
                (1f32, 1f32, 1f32, 1f32),
            );
            let color = target_manager.color.attachments[0].clone();
            let rect = RectF::with_size(color.width as f32, color.height as f32);
            batch.tex(
                &(rect + PointF::new(GAME_PIXEL_WIDTH as f32 * 3f32, GAME_PIXEL_HEIGHT as f32)),
                color,
                (1f32, 1f32, 1f32, 1f32),
            );
            let game = target_manager.game.attachments[0].clone();
            let rect = RectF::with_size(game.width as f32, game.height as f32);
            batch.tex(
                &(rect + PointF::new(GAME_PIXEL_WIDTH as f32 * 3f32, 0f32)),
                game,
                (1f32, 1f32, 1f32, 1f32),
            );
        } else {
            // Draw map
            let matrix = glm::translation(&glm::vec3(self.offset.0, self.offset.1, 0.0f32));
            let scale = glm::scaling(&glm::vec3(self.zoom, self.zoom, 0.0f32));
            batch.push_matrix(scale * matrix);

            for room in Content::map().rooms.iter() {
                batch.sprite(&room.rect, &room.albedo(), (1f32, 1f32, 1f32, 1f32));
            }
            batch.pop_matrix();

            // Debug::image(texture_id, size);

            // Draw editor guidelines, should this be a shader?
            for i in -15..15 {
                let mut guide = RectF::with_size(1f32, SCREEN_HEIGHT as f32);
                guide.translate_by(&PointF::new(
                    (self.offset.0 + (i as f32 * (GAME_PIXEL_WIDTH as f32))) * self.zoom,
                    0f32,
                ));
                if (i == 0) {
                    batch.rect(&guide, (1f32, 1f32, 1f32, 1.0f32));
                } else {
                    batch.rect(&guide, (1f32, 1f32, 1f32, 0.25f32));
                }
            }
            for i in -15..15 {
                let mut guide = RectF::with_size(SCREEN_WIDTH as f32, 1f32);
                guide.translate_by(&PointF::new(
                    0f32,
                    (self.offset.1 + (i as f32 * (GAME_PIXEL_HEIGHT as f32))) * self.zoom,
                ));
                if (i == 0) {
                    batch.rect(&guide, (1f32, 1f32, 1f32, 1.0f32));
                } else {
                    batch.rect(&guide, (1f32, 1f32, 1f32, 0.25f32));
                }
            }

            // Draw hover pixel indicator
            if (self.hover_tile.is_some()) {
                let hover_tile = self.hover_tile.as_ref().unwrap();
                batch.rect(hover_tile, (1f32, 1f32, 1f32, 0.5f32));
            }
        }
    }

    // Utils
    fn screen_to_world(&self, screen: (i32, i32)) -> (i32, i32) {
        let world = (
            (screen.0 as f32 / self.zoom) - self.offset.0 as f32,
            (screen.1 as f32 / self.zoom) - self.offset.1 as f32,
        );
        (world.0.round() as i32, world.1.round() as i32)
    }
    fn world_to_screen(&self, world: (i32, i32)) -> (i32, i32) {
        let screen = (
            (world.0 as f32 + self.offset.0) * self.zoom,
            (world.1 as f32 + self.offset.1) * self.zoom,
        );
        (screen.0.round() as i32, screen.1.round() as i32)
    }
    fn world_to_room(room: &Room, world_pos: (i32, i32)) -> (i32, i32) {
        let x = world_pos.0 - room.world_position.x as i32;
        let y = world_pos.1 - room.world_position.y as i32;
        (x, y)
    }
    fn room_to_world(room: &Room, room_pos: (i32, i32)) -> (i32, i32) {
        let x = room_pos.0 + room.world_position.x as i32;
        let y = room_pos.1 + room.world_position.y as i32;
        (x, y)
    }
}
