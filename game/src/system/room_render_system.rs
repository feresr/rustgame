use common::Debug;
use engine::graphics::batch::Batch;

use crate::{
    content::Content,
    target_manager::{self, TargetManager},
};

pub struct RoomRenderSystem {}

// Re renders maps with stale (dirty) into maps_color (large pre-rendered map texture)
impl RoomRenderSystem {
    pub fn render(batch : &mut Batch, target_manager: &TargetManager) {
        batch.clear();
        let map = Content::map();
        for room in map.rooms.iter_mut().filter(|r| r.is_dirty) {
            room.render_colors_into(batch);
        }
        batch.render(&target_manager.maps_color); // Render all maps in one go
        batch.clear();

        for room in map.rooms.iter_mut().filter(|r| r.is_dirty) {
            room.render_normals_into(batch);
            room.is_dirty = false;
        }

        batch.render(&target_manager.maps_normal); // Render all maps in one go
        batch.clear();

        Debug::window("Map textures");

        // let tileset = Content::get().tilesets.get(&0).unwrap();
        // Debug::image(tileset.normal.id as usize);

        let color = target_manager.maps_color.color();
        Debug::image(0, color.id as usize, (color.width as f32 / 2f32, color.height as f32 / 2f32));
        let color = target_manager.maps_normal.color();
        Debug::image(0, color.id as usize, (color.width as f32 / 2f32, color.height as f32 / 2f32));
        let color = target_manager.maps_outline.color();
        Debug::image(0, color.id as usize, (color.width as f32 / 2f32, color.height as f32 / 2f32));
    }
}
