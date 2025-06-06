use engine::graphics::{target::Target, texture::TextureFormat};

use crate::game_state::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, ROOM_COUNT_H, ROOM_COUNT_W, SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct TargetManager {
    pub screen: Target,       // Final composited image presented on screen
    
    pub lights: Target, // Lightmap texture (grayscale): white = lit, black = shadow
    pub color: Target, // Result of albedo * lighting, used for final color pass
    // The two targets end up combined into 'game'
    pub game: Target, // Render target at GAME_PIXEL_SIZE; to be scaled up to 'screen' size for pixel-perfect display
    
    pub maps_color: Target, // Combined albedo textures for all map layers
    pub maps_normal: Target, // Combined normal maps for all map layers
    pub maps_outline: Target, // Combined normal maps for all map layers
}

impl TargetManager {
    pub fn new() -> Self {
        TargetManager {
            screen: Target::screen(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
            game: Target::new(
                GAME_PIXEL_WIDTH as i32,
                GAME_PIXEL_HEIGHT as i32,
                &[TextureFormat::RGBA],
            ),
            maps_color: Target::new(
                (GAME_PIXEL_WIDTH * ROOM_COUNT_W) as i32,
                (GAME_PIXEL_HEIGHT * ROOM_COUNT_H) as i32,
                &[TextureFormat::RGBA],
            ),
            maps_normal: Target::new(
                (GAME_PIXEL_WIDTH * ROOM_COUNT_W) as i32,
                (GAME_PIXEL_HEIGHT * ROOM_COUNT_H) as i32,
                &[TextureFormat::RGBA],
            ),
            maps_outline: Target::new(
                (GAME_PIXEL_WIDTH * ROOM_COUNT_W) as i32,
                (GAME_PIXEL_HEIGHT * ROOM_COUNT_H) as i32,
                &[TextureFormat::RGBA],
            ),
            lights: Target::new(
                GAME_PIXEL_WIDTH as i32,
                GAME_PIXEL_HEIGHT as i32,
                &[TextureFormat::RGBA, TextureFormat::DepthStencil],
            ),
            color: Target::new(
                GAME_PIXEL_WIDTH as i32,
                GAME_PIXEL_HEIGHT as i32,
                &[TextureFormat::RGBA],
            ),
        }
    }
}
