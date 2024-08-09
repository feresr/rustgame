use engine::{
    ecs::{Component, RenderWorld, UpdateWorld},
    graphics::{
        batch::Batch,
        common::RectF,
        target::Target,
        texture::{Texture, TextureFormat},
    },
};
use rand::Rng;

use crate::{GAME_HEIGHT, GAME_WIDTH, TILE_SIZE};

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    SOLID,
    EMPTY,
}

pub struct Room {
    pub tiles: [Tile; GAME_WIDTH * GAME_HEIGHT],
    pub rect: RectF,
    texture: Option<Texture>,
    ortho: glm::Mat4,
    translation_matrix: glm::Mat4,
}
impl Room {
    pub fn new() -> Self {
        let mut rand = rand::thread_rng();
        let mut tiles = [Tile::EMPTY; GAME_HEIGHT * GAME_WIDTH];

        let mut index = 0;
        for i in tiles {
            let r = rand.r#gen::<f32>();
            if r > 0.97 {
                tiles[index] = Tile::SOLID
            }
            index += 1;
        }
        let rect = RectF {
            x: 0.0,
            y: 0.0,
            w: GAME_WIDTH as f32,
            h: GAME_HEIGHT as f32,
        };
        Room {
            tiles,
            rect,
            texture: None,
            translation_matrix: glm::Mat4::new_translation(&glm::vec3(0.0f32, 0.0f32, -0.3f32)),
            ortho: glm::ortho(
                0.0,
                GAME_WIDTH as f32,
                0 as f32,
                GAME_HEIGHT as f32,
                -1.0,
                1.0,
            ),
        }
    }
    pub fn at(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x + y * GAME_WIDTH]
    }
    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[x + y * GAME_WIDTH] = tile;
    }
}
impl Component for Room {
    fn update<'a>(&mut self, world: &'a mut UpdateWorld<'_>, entity: u32) {}

    fn render<'a>(&mut self, world: &'a mut RenderWorld<'_>, batch: &mut Batch, entity: u32) {
        if let None = self.texture {
            let attachments = [TextureFormat::RGBA, TextureFormat::DepthStencil];
            let target = Target::new(GAME_WIDTH as i32, GAME_HEIGHT as i32, &attachments);
            let mut batch = Batch::default();

            let mut x = 0;
            let mut y = 0;
            for tile in self.tiles.iter() {
                match tile {
                    Tile::SOLID => {
                        let tile_rect = RectF {
                            x: (x * TILE_SIZE) as f32,
                            y: (y * TILE_SIZE) as f32,
                            w: TILE_SIZE as f32,
                            h: TILE_SIZE as f32,
                        };

                        let mut rand = rand::thread_rng();
                        let r: f32 = rand.r#gen();
                        let g: f32 = rand.r#gen();
                        let b: f32 = rand.r#gen();
                        batch.rect(&tile_rect, (r, g, b));
                    }
                    Tile::EMPTY => {}
                }
                if x >= GAME_WIDTH - 1 {
                    x = 0;
                    y += 1;
                } else {
                    x += 1;
                }
            }
            batch.render(&target, &self.ortho);
            self.texture = Some(*target.attachments.get(0).unwrap());
        }
        batch.push_matrix(self.translation_matrix);
        batch.tex(&self.rect, &self.texture.unwrap(), (0.0, 0.0, 0.0));
        batch.pop_matrix();
    }
}
