mod components;
mod scene;

extern crate engine;
extern crate nalgebra_glm as glm;

use components::{
    background::Background,
    ball::Ball,
    collider::{Collider, ColliderType},
    mover::Mover,
    player::Player,
    position::Position,
    room::{Room, Tile},
};

use engine::{
    ecs::{Component, Entity, World, WorldOp},
    graphics::{batch::*, common::*, material::Material, shader::*, target::*, texture::*},
    Config, Game,
};
use glm::Vec3;
use imgui::Ui;
use scene::{GameScene, Scene};
use std::env;

const SCREEN_WIDTH: usize = GAME_PIXEL_WIDTH * 4;
const SCREEN_HEIGHT: usize = GAME_PIXEL_HEIGHT * 4;
const SCREEN_RESOLUTION: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const TILE_SIZE: usize = 8;
const GAME_PIXEL_WIDTH: usize = 320;
const GAME_PIXEL_HEIGHT: usize = 184;

const GAME_TILE_WIDTH: usize = GAME_PIXEL_WIDTH / TILE_SIZE;
const GAME_TILE_HEIGHT: usize = GAME_PIXEL_HEIGHT / TILE_SIZE;

struct Brick {
    rect: RectF,
    color: Vec3,
}
impl Component for Brick {
    fn render<'a>(&mut self, _entity: Entity<'a, impl WorldOp>, batch: &mut Batch) {
        let brick = self;
        batch.rect(&brick.rect, (brick.color.x, brick.color.y, brick.color.z));
    }
}

struct Gravity {
    pub value: f32,
}
impl Component for Gravity {}

struct Camera {
    target: glm::Vec3,
    pos: glm::Vec3,
    dir: glm::Vec3,
    right: glm::Vec3,
    up: glm::Vec3,
}
const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec3 aPos;\n
            layout (location = 1) in vec3 aColor;\n
            layout (location = 2) in vec2 aTexCoord;\n
            uniform mat4 u_matrix;\n
            out vec2 TexCoord;\n
            out vec3 a_color;\n
            void main()\n
            {\n
               gl_Position = u_matrix * vec4(aPos, 1.0);\n
               TexCoord = aTexCoord;
               a_color = aColor;
            }";
const FRAGMENT_SHADER_SOURCE_2: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            out vec4 FragColor;\n
            uniform sampler2D u_texture;\n
            uniform ivec2 u_resolution;\n
            uniform float offset;
            uniform float radius;
            uniform float time;
            void main()\n
            {\n
                vec2 c = (2.0 * gl_FragCoord.xy - u_resolution.xy) / u_resolution.x; 
                c = c * 25.0;
                if (length(sin(c + vec2(offset,offset))) < radius) {
                    FragColor = vec4(0.8);
                } else {
                    FragColor = vec4(min(0.1, sin(time)), min(0.2, cos(time)), min(0.3, sin(time * 2.0)), 1.0);
                }
            }";

struct Foo {
    world: World,
    ortho: glm::Mat4,
    screen_ortho: glm::Mat4,
    target: Target,
    current_scene: Box<dyn Scene>,
}

struct FrameBuffers {
    index: u32,
}

impl Game for Foo {
    fn init(&mut self) {
        self.current_scene.init();
        self.world.add_resource(FrameBuffers { index: 123 });

        let pos = glm::vec3(0.0, 0.0, 1.0);
        let target = glm::vec3(0.0, 0.0, 0.0);
        let dir = glm::normalize(&(target - pos));
        let up = glm::vec3(0.0, 1.0, 0.0);
        let right = glm::normalize(&(glm::cross(&up, &dir)));
        let camera_up = glm::cross(&dir, &right);

        let camera = Camera {
            target,
            pos,
            dir,
            right,
            up: camera_up,
        };

        let view = glm::look_at(&camera.pos, &(camera.pos + camera.dir), &camera.up);
        // Background is at z 0
        // Camera is at z 1 - Looking at 0
        let ortho: glm::Mat4 = glm::ortho(
            0.0,
            GAME_PIXEL_WIDTH as f32,
            0f32,
            GAME_PIXEL_HEIGHT as f32,
            0.0f32,
            2f32,
        );
        self.ortho = ortho * view;

        self.screen_ortho = glm::ortho(
            0.0,
            SCREEN_WIDTH as f32,
            0f32,
            SCREEN_HEIGHT as f32,
            0.0f32,
            2f32,
        );

        let room = Room::from_path("src/map.png");
        let mut room_entity = self.world.add_entity();
        room_entity.assign(Collider::new(ColliderType::Grid {
            columns: GAME_TILE_WIDTH,
            rows: GAME_TILE_HEIGHT,
            tile_size: TILE_SIZE,
            cells: room.tiles.map(|f| f == Tile::SOLID).to_vec(),
        }));
        room_entity.assign(room);

        // Shaders
        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        let mut bkg = self.world.add_entity();
        let border = 20f32;
        bkg.assign(Background {
            offset: 1.2,
            radius: 0.20,
            time: 0.0,
            material: Material::new(shader),
            rect: RectF {
                x: border,
                y: border,
                w: (320 - 2 * border as i32) as f32,
                h: (180 - 2 * border as i32) as f32,
            },
            translation_matrix: glm::Mat4::new_translation(&glm::vec3(0.0, 0.0, -0.2f32)),
        });
        let mut ball = self.world.add_entity();
        ball.assign(Ball {
            r: 2,
            spawned_a_new: 1884,
        });
        ball.assign(Mover::new(1.0, 1.0));
        ball.assign(Position::new(
            GAME_PIXEL_WIDTH as i32 / 2,
            GAME_PIXEL_HEIGHT as i32 / 2,
        ));
        // ball.assign(Gravity { value: 0f32 });
        ball.assign(Collider::new(ColliderType::Rect {
            rect: RectF::with_size(2f32, 2f32),
        }));

        let mut paddle = self.world.add_entity();
        paddle.assign(Player::new(8, 8, Texture::from_path("src/blob.png")));
        paddle.assign(Mover::default());
        paddle.assign(Collider::new(ColliderType::Rect {
            rect: RectF {
                x: 1.0,
                y: 0.0,
                w: 6.0,
                h: 6.0,
            },
        }));
        paddle.assign(Position::default());
        paddle.assign(Gravity { value: 0.7f32 });

        // let brick_size = vec2(10f32, 4f32);
        // let gap = vec2(10f32, 10f32);
        // for x in 0..12 {
        //     for y in 0..10 {
        //         let mut brick = self.world.add_entity();
        //         brick.assign(Brick {
        //             rect: RectF {
        //                 x: 40 as f32 + (x as f32) * (brick_size.x + gap.x),
        //                 y: 150 as f32 - (y as f32 * (brick_size.y + gap.y)) as f32,
        //                 w: brick_size.x,
        //                 h: brick_size.y,
        //             },
        //             color: vec3(x as f32 / 12f32, 1f32 - x as f32 / 12f32, y as f32 / 12f32),
        //         });
        //     }
        // }
        let attachments = [TextureFormat::RGBA, TextureFormat::DepthStencil];
        self.target = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &attachments,
        );
    }

    fn update(&mut self) -> bool {
        self.current_scene.update();
        self.world.update();
        return true;
    }

    fn render(&self, batch: &mut Batch) {
        self.current_scene.render(batch);
        {
            // Render into low-res target
            self.target.clear((0f32, 0.1f32, 0.2f32));
            self.world.render(batch);
            batch.set_sampler(&TextureSampler::nearest());
            batch.render(&self.target, &self.ortho);
            batch.clear();
        }
        {
            //Render low-res target to screen
            // TODO: do not create rect here.
            let SCREEN = Target::screen(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            let f = RectF::with_size(SCREEN.width as f32, SCREEN.height as f32);
            batch.set_sampler(&TextureSampler::nearest());
            batch.tex(&f, &self.target.attachments[0], (0f32, 0f32, 0f32));
            batch.render(&SCREEN, &self.screen_ortho);
        }
    }

    fn dispose(&mut self) {}

    fn debug(&self, imgui: &Ui) {
        self.world.debug(imgui);
    }

    fn config(&self) -> engine::Config {
        Config {
            window_width: SCREEN_WIDTH as u32,
            window_height: SCREEN_HEIGHT as u32,
        }
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let game = Foo {
        world: World::new(),
        ortho: glm::Mat4::identity(),
        screen_ortho: glm::Mat4::identity(),
        target: Target::empty(),
        current_scene: Box::new(GameScene::with_map("src/map.png")),
    };
    engine::run(game);
}
