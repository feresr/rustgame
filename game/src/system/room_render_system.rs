use common::Debug;
use engine::graphics::batch::Batch;

use crate::{
    content::Content,
    target_manager::{self, TargetManager},
};

pub struct RoomRenderSystem {}

// Re renders maps with stale (dirty) into maps_color (large pre-rendered map texture)
impl RoomRenderSystem {
    pub fn render(batch: &mut Batch, target_manager: &TargetManager) {
        batch.clear();

        let map = Content::map();
        let color_target = &target_manager.maps_color;
        for room in map.rooms.iter_mut().filter(|r| r.is_dirty) {
            room.prerender_colors(batch);
            room.set_color_texture(color_target.color());
        }
        batch.render(color_target);
        batch.clear();

        let normal_target = &target_manager.maps_normal;
        for room in map.rooms.iter_mut().filter(|r| r.is_dirty) {
            room.prerender_normals(batch);
            room.set_normal_texture(normal_target.color());
        }
        batch.render(normal_target);
        batch.clear();

        for room in map.rooms.iter_mut().filter(|r| r.is_dirty) {
            room.is_dirty = false;
        }

        Debug::window("Map textures");

        // let tileset = Content::get().tilesets.get(&0).unwrap();
        // Debug::image(tileset.normal.id as usize);

        let color = target_manager.maps_color.color();
        Debug::image(color.id as usize, (color.width as f32 / 2f32, color.height as f32 / 2f32));
        let color = target_manager.maps_normal.color();
        Debug::image(color.id as usize, (color.width as f32 / 2f32, color.height as f32 / 2f32));
        let color = target_manager.maps_outline.color();
        Debug::image(color.id as usize, (color.width as f32 / 2f32, color.height as f32 / 2f32));
    }
}
