use engine::{ecs::component::Component, graphics::{
    batch::Batch,
    common::RectF,
    target::Target,
    texture::{Texture, TextureFormat},
}};

use rand::Rng;
use std::{fs::File, io::Read};

use crate::{GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH, GAME_TILE_HEIGHT, GAME_TILE_WIDTH, TILE_SIZE};

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    SOLID,
    EMPTY,
}

#[derive(Clone)]
pub struct Room {
    pub tiles: [Tile; GAME_TILE_WIDTH * GAME_TILE_HEIGHT],
    pub rect: RectF,
    texture: Option<Texture>,
    ortho: glm::Mat4,
    translation_matrix: glm::Mat4,
}
impl Room {
    pub fn from_path(path: &str) -> Self {
        // Load file into memory
        print!("Creating Room from path path {}", path);
        let mut f = File::open(path).expect("File not found: ");
        let mut contents = vec![];
        f.read_to_end(&mut contents).unwrap();

        // Load the image
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut comp: i32 = 0;
        let img: *mut u8;
        unsafe {
            stb_image_rust::stbi_set_flip_vertically_on_load(1);
            img = stb_image_rust::stbi_load_from_memory(
                contents.as_mut_ptr(),
                contents.len() as i32,
                &mut x,
                &mut y,
                &mut comp,
                stb_image_rust::STBI_rgb_alpha,
            );
        }
        assert!(
            x == GAME_TILE_WIDTH as i32,
            "Map texture width must be GAME_TILE_WIDTH"
        );
        assert!(
            y == GAME_TILE_HEIGHT as i32,
            "Map texture height must be GAME_TILE_HEIGHT"
        );

        let mut tiles = [Tile::EMPTY; GAME_TILE_WIDTH * GAME_TILE_HEIGHT];
        unsafe {
            for i in 0..tiles.len() {
                if *img.add(i * 4 + 3) > 0 {
                    tiles[i] = Tile::SOLID
                }
            }
        }
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
            translation_matrix: glm::Mat4::new_translation(&glm::vec3(0.0f32, 0.0f32, -0.2f32)),
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
    pub fn new_random() -> Self {
        let mut rand = rand::thread_rng();
        let mut tiles = [Tile::EMPTY; GAME_TILE_WIDTH * GAME_TILE_HEIGHT];

        let mut index = 0;
        for i in tiles {
            let r = rand.r#gen::<f32>();
            if r > 0.76 {
                tiles[index] = Tile::SOLID
            }
            index += 1;
        }
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
            translation_matrix: glm::Mat4::new_translation(&glm::vec3(0.0f32, 0.0f32, -0.3f32)),
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
    pub fn at(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x + y * GAME_TILE_WIDTH]
    }
    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[x + y * GAME_TILE_WIDTH] = tile;
    }
}
impl Component for Room {
    fn render<'a>(
        &mut self,
        _entity: engine::ecs::Entity<'a, impl engine::ecs::WorldOp>,
        batch: &mut Batch,
    ) {
        if let None = self.texture {
            // Render the Room into a texture only once. Then re-render that texture into the game buffer.
            let attachments = [TextureFormat::RGBA, TextureFormat::DepthStencil];
            let target = Target::new(
                GAME_PIXEL_WIDTH as i32,
                GAME_PIXEL_HEIGHT as i32,
                &attachments,
            );
            target.clear((0.0f32, 0.0f32, 0.0f32));
            // Creates a new batch (we don't want to clear the current content of the game batch - we need to actually draw these)
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
                        batch.rect(&tile_rect, (rand.r#gen(), rand.r#gen(), rand.r#gen()));
                    }
                    Tile::EMPTY => {}
                }
                if x >= GAME_TILE_WIDTH - 1 {
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
