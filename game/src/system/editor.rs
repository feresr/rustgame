use crate::game_state;
use glm::Vec2;
use imgui::{Condition, Ui};

use common::{Debug, Keyboard, Mouse};
use engine::{
    graphics::{
        batch::Batch,
        common::{PointF, RectF},
    },
    Keycode,
};

use crate::components::room::Tile;
use crate::game_state::{GAME_TILE_HEIGHT, GAME_TILE_WIDTH};
use crate::{
    components::room::Room,
    content::Content,
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH},
    target_manager::TargetManager,
};

static mut DEBUG: Option<DebugInfo> = None;

#[derive(Debug, Default)]
pub struct DebugInfo {
    world_mouse: (i32, i32),
    room_mouse: (i32, i32),
    current_room_world_position: Vec2,
    current_selected_tile: (i32, i32),
}

pub struct Editor {
    selection: Option<&'static mut Room>,
    debug_textures: bool,
    zoom: f32,
    offset: (f32, f32),
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            selection: None,
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
        unsafe {
            if DEBUG.is_none() {
                DEBUG = Some(DebugInfo::default());
            }
        }
        let mut debug = unsafe { &mut DEBUG.as_mut().unwrap() };
        debug.world_mouse = world_mouse;

        let room = Content::map().rooms.iter().find(|room| {
            room.rect
                .contains(&PointF::new(world_mouse.0 as f32, world_mouse.1 as f32))
        });
        if room.is_some() {
            let room = room.unwrap();
            let room_mouse = (
                world_mouse.0 - room.world_position.x as i32,
                world_mouse.1 - room.world_position.y as i32,
            );
            debug.room_mouse = room_mouse;
            debug.current_room_world_position = room.world_position.clone_owned();

            let selected_tile_x =
                (room_mouse.0 as f32 / GAME_PIXEL_WIDTH as f32) * GAME_TILE_WIDTH as f32;
            let selected_tile_y =
                (room_mouse.1 as f32 / GAME_PIXEL_HEIGHT as f32) * GAME_TILE_HEIGHT as f32;
            debug.current_selected_tile = (selected_tile_x as i32, selected_tile_y as i32)
        }

        if Mouse::left_held() {
            for room in Content::map().rooms.iter_mut() {
                if room
                    .rect
                    .contains(&PointF::new(world_mouse.0 as f32, world_mouse.1 as f32))
                {
                    let first_layer = room.layers.first_mut().unwrap();
                    let room_mouse = (
                        world_mouse.0 - room.world_position.x as i32,
                        world_mouse.1 - room.world_position.y as i32,
                    );

                    let selected_tile_x =
                        (room_mouse.0 as f32 / GAME_PIXEL_WIDTH as f32) * GAME_TILE_WIDTH as f32;
                    let selected_tile_y =
                        (room_mouse.1 as f32 / GAME_PIXEL_HEIGHT as f32) * GAME_TILE_HEIGHT as f32;
                    if let Tile::Solid { .. } =
                        first_layer.tiles[selected_tile_y as usize][selected_tile_x as usize]
                    {
                        // first_layer.tiles[selected_tile_y as usize][selected_tile_x as usize] = Tile::Empty {};
                    } else {
                        first_layer.tiles[selected_tile_y as usize][selected_tile_x as usize] =
                            Tile::Solid {
                                src_x: 0,
                                src_y: 0,
                                kind: 0,
                            };
                    }
                }
            }
        }

        if !Mouse::left_held() {
            if (self.selection.is_some()) {
                // Drop selection in to a valid square
                let room = self.selection.as_mut().unwrap();

                room.rect.x =
                    ((world_mouse.0 as usize / GAME_PIXEL_WIDTH) * GAME_PIXEL_WIDTH) as f32;
                room.rect.y =
                    ((world_mouse.1 as usize / GAME_PIXEL_HEIGHT) * GAME_PIXEL_HEIGHT) as f32;
            }
            self.selection = None;
        }

        // The rooms are not panned around
        // if self.selection.is_none() {
        //     for room in Content::map().rooms.iter_mut() {
        //         if room
        //             .rect
        //             .contains(&PointF::new(world_mouse.0 as f32, world_mouse.1 as f32))
        //         {
        //             // Select
        //             if Mouse::left_pressed() {
        //                 self.selection = Some(room);
        //             }
        //         }
        //     }
        // } else {
        //     // Move
        //     self.selection
        //         .as_mut()
        //         .unwrap()
        //         .rect
        //         .translate_by(&PointF::new(world_mouse_rel.0, world_mouse_rel.1));
        // }

        // Zoom: https://www.youtube.com/watch?v=ZQ8qtAizis4
        if Mouse::wheel().1 != 0 && self.selection.is_none() {
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
        if Mouse::right_held() && self.selection.is_none() {
            self.offset.0 += world_mouse_rel.0;
            self.offset.1 += world_mouse_rel.1;
        }
    }

    fn screen_to_world(&self, screen: (i32, i32)) -> (i32, i32) {
        let world = (
            (screen.0 as f32 / self.zoom) - self.offset.0 as f32,
            (screen.1 as f32 / self.zoom) - self.offset.1 as f32,
        );
        (world.0.round() as i32, world.1.round() as i32)
    }

    pub fn render(&mut self, batch: &mut Batch, target_manager: &TargetManager) {
        game_state().refresh();
        batch.clear();

        Debug::window("Map editor");
        Debug::display(&format!("Zoom level {:.1}", self.zoom));
        Debug::separator();
        let mouse_pos = Mouse::position();
        Debug::display(
        &format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos.0, mouse_pos.1)
        );

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
            // This should be a shader?
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

            // Draw map
            let matrix = glm::translation(&glm::vec3(self.offset.0, self.offset.1, 0.0f32));
            let scale = glm::scaling(&glm::vec3(self.zoom, self.zoom, 0.0f32));
            batch.push_matrix(scale * matrix);

            if let Some(selection) = &self.selection {
                let mut outter = selection.rect.clone();
                outter.scale(1.1);
                batch.rect(&outter, (1f32, 1f32, 1f32, 1.0f32));
                let mut inner = selection.rect.clone();
                inner.scale(1.05);
                batch.rect(&inner, (0f32, 0f32, 0f32, 1.0f32));
            }
            for room in Content::map().rooms.iter() {
                batch.sprite(&room.rect, &room.albedo(), (1f32, 1f32, 1f32, 1f32));
            }
            batch.pop_matrix();
        }
    }
}
