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
    pub src_x: i64,
    pub src_y: i64,
    pub kind: u32,
}

pub struct Layer {
    tileset_id: i64,
    pub tiles: Vec<Tile>,
}

pub struct Room {
    pub layers: Vec<Layer>,
    pub rect: RectF,
    pub texture: Option<Texture>,
    pub ortho: glm::Mat4,
    batch: Batch,
}
impl Room {
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
                        tileset_id: layer.tileset_def_uid.expect("Missing tileset id"),
                        tiles,
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
            x: 0.0,
            y: 0.0,
            w: GAME_PIXEL_WIDTH as f32,
            h: GAME_PIXEL_HEIGHT as f32,
        };
        Room {
            layers,
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
            batch: Batch::default(),
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

        for layer in self.layers.iter().rev() {
            let tileset = content().tilesets.get(&layer.tileset_id).unwrap();
            for tile in layer.tiles.iter() {
                let tile_rect = RectF {
                    x: tile.x as f32,
                    y: tile.y as f32,
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                };
                self.batch.sprite(
                    &tile_rect,
                    &SubTexture::new(
                        &tileset.texture,
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
            self.batch.render(&target, &self.ortho);
        }

        self.texture = Some(*target.color());
        self.batch.clear();
    }
}
impl Component for Room {
    const CAPACITY: usize = 8;
}
