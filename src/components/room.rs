use crate::{content::content, GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, TILE_SIZE};
use engine::{
    ecs::component::Component,
    graphics::{
        batch::Batch,
        common::RectF,
        target::Target,
        texture::{SubTexture, Texture, TextureFormat},
    },
};
use ldtk_rust::Level;

#[derive(Clone, Copy, PartialEq)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub kind: u32,
}

#[derive(Clone)]
pub struct Room {
    pub tiles: Vec<Tile>,
    pub rect: RectF,
    pub texture: Option<Texture>,
    ortho: glm::Mat4,
}
impl Room {
    pub fn from_level(level: &Level) -> Self {
        // Load file into memory

        // let mut tiles = [Tile::EMPTY; GAME_TILE_WIDTH * GAME_TILE_HEIGHT];
        let layer = level.layer_instances.as_ref().unwrap().first().unwrap();

        assert!(
            level.px_wid == GAME_PIXEL_WIDTH as i64,
            "Level width must be GAME_PIXEl_WIDTH"
        );
        assert!(
            level.px_hei == GAME_PIXEL_HEIGHT as i64,
            "Level width must be GAME_PIXEL_HEIGHT"
        );

        let tiles = layer
            .grid_tiles
            .iter()
            .map(|f| Tile {
                x: f.px[0] as u32,
                y: f.px[1] as u32,
                kind: f.t as u32,
            })
            .collect();

        let rect = RectF {
            x: 0.0,
            y: 0.0,
            w: GAME_PIXEL_WIDTH as f32,
            h: GAME_PIXEL_HEIGHT as f32,
        };
        Room {
            tiles,
            rect,
            texture: None,
            ortho: glm::ortho(
                0.0,
                GAME_PIXEL_WIDTH as f32,
                0 as f32,
                GAME_PIXEL_HEIGHT as f32,
                -1.0,
                1.0,
            ),
        }
    }
    pub fn prerender(&mut self) {
        let attachments = [TextureFormat::RGBA];
        let target = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &attachments,
        );
        target.clear((1.0f32, 0.0f32, 1.0f32, 0f32));
        // Creates a new batch (we don't want to clear the current content of the game batch - we need to actually draw these)
        let mut batch = Batch::default();
        let atlas = content().altas();
        for tile in self.tiles.iter() {
            let tile_rect = RectF {
                x: tile.x as f32,
                y: tile.y as f32,
                w: TILE_SIZE as f32,
                h: TILE_SIZE as f32,
            };

            let xy = match tile.kind {
                0 => (0f32, 0f32),
                1 => (8f32, 0f32),
                2 => (0f32, 8f32),
                3 => (8f32, 8f32),
                _ => (0f32, 0f32),
            };
            batch.sprite(
                &tile_rect,
                &SubTexture::new(
                    atlas,
                    RectF {
                        x: xy.0,
                        y: xy.1,
                        w: 8f32,
                        h: 8f32,
                    },
                ),
                (1f32, 1f32, 1f32),
            );
        }
        batch.render(&target, &self.ortho);
        self.texture = Some(*target.color());
    }
}
impl Component for Room {
    const CAPACITY: usize = 8;
}
