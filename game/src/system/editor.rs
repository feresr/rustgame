use engine::graphics::{
    batch::Batch,
    common::{PointF, RectF},
};

use crate::{
    game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH},
    target_manager::TargetManager,
};

pub struct Editor {}
impl Default for Editor {
    fn default() -> Self {
        Editor {}
    }
}

impl Editor {
    pub fn render(&mut self, batch: &mut Batch, target_manager: &TargetManager) {
        batch.clear();
        let maps = target_manager.maps_normal.attachments[0].clone();
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
        // content()
        //     .map
        //     .rooms
        //     .iter_mut()
        //     .filter_map(|room| room.as_mut())
        //     .for_each(|room| {
        //         room.prerender(batch);
        //         batch.clear();
        //         let room_texture = room.albedo();
        //         batch.tex(&room.rect, room_texture, (1f32, 1f32, 1f32, 1f32));
        //     });
    }
}
