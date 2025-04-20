use std::{f32::NAN, u64::MAX};

use common::{Keyboard, Mouse};
use engine::{
    graphics::{
        batch::Batch,
        common::{PointF, RectF},
    },
    Keycode,
};
use glm::{lerp, scale, translate2d};
use sdl2::{libc::VEOL, sys::__FLOAT_WORD_ORDER};

use crate::{
    components::room::Room,
    content::{self, Content},
    game_state::{self, GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH},
    target_manager::TargetManager,
};

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

        if !Mouse::left_held() {
            self.selection = None;
        }

        // The rooms are not panned around
        if self.selection.is_none() {
            for room in Content::map().rooms.iter_mut().flatten() {
                if room
                    .rect
                    .contains(&PointF::new(world_mouse.0 as f32, world_mouse.1 as f32))
                {
                    // Select
                    if Mouse::left_pressed() {
                        self.selection = Some(room);
                    }
                }
            }
        } else {
            // Move
            let rel = Mouse::position_rel();
            self.selection
                .as_mut()
                .unwrap()
                .rect
                .translate_by(&PointF::new(world_mouse_rel.0, world_mouse_rel.1));
        }

        // Zoom: https://www.youtube.com/watch?v=ZQ8qtAizis4
        if Mouse::wheel().1 != 0 && self.selection.is_none() {
            let screen_position = Mouse::position();
            let before_zoom_world_position = self.screen_to_world(screen_position);
            self.zoom += Mouse::wheel().1 as f32 * 0.1;
            self.zoom = self.zoom.clamp(0.25, 2.5);
            let after_zoom_world_position = self.screen_to_world(screen_position);

            self.offset.0 +=
                after_zoom_world_position.0 as f32 - before_zoom_world_position.0 as f32;
            self.offset.1 +=
                after_zoom_world_position.1 as f32 - before_zoom_world_position.1 as f32;
        }
        if Mouse::left_held() && self.selection.is_none() {
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
            for room in Content::map().rooms.iter().flatten() {
                batch.sprite(&room.rect, &room.albedo(), (1f32, 1f32, 1f32, 1f32));
            }
            batch.pop_matrix();
        }
    }
}
