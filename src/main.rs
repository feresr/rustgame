mod components;
mod player;
mod scene;

extern crate engine;
extern crate nalgebra_glm as glm;

use components::{
    background::Background,
    ball::Ball,
    collider::{Collider, ColliderType},
    controller::Controller,
    mover::Mover,
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
use player::Player;
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

struct Map {
    height: usize,
    width: usize,
    room_names: Vec<Option<String>>,
}
impl Map {
    fn new(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            room_names: vec![None; width * height],
        }
    }
    fn set(&mut self, x: usize, y: usize, path: String) {
        assert!(x < self.width);
        assert!(y < self.height);
        self.room_names[x + y * self.width] = Some(path);
    }
    fn get(&self, x: usize, y: usize) -> Option<String> {
        assert!(x < self.width);
        assert!(y < self.height);
        if let Some(s) = &self.room_names[x + y * self.width] {
            return Some(s.to_owned());
        }
        return None;
    }
}

struct Foo {
    ortho: glm::Mat4,
    screen_ortho: glm::Mat4,
    target: Target,
    scene: GameScene,
    map: Map,
    current_room: (usize, usize),
}
impl Foo {
    fn new() -> Self {
        let mut map = Map::new(2, 2);
        map.set(0, 0, "src/map.png".to_owned());
        map.set(1, 0, "src/map2.png".to_owned());
        map.set(0, 1, "src/map3.png".to_owned());
        map.set(1, 1, "src/map4.png".to_owned());

        let first_room = (0, 0);
        let first = map.get(first_room.0, first_room.1).unwrap();
        Self {
            ortho: glm::Mat4::identity(),
            screen_ortho: glm::Mat4::identity(),
            target: Target::empty(),
            scene: GameScene::with_map(first),
            map,
            current_room: first_room,
        }
    }
}

impl Game for Foo {
    fn init(&mut self) {
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

        // Shaders
        // let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        // let mut bkg = self.world.add_entity();
        // let border = 20f32;
        // bkg.assign(Background {
        //     offset: 1.2,
        //     radius: 0.20,
        //     time: 0.0,
        //     material: Material::new(shader),
        //     rect: RectF {
        //         x: border,
        //         y: border,
        //         w: (320 - 2 * border as i32) as f32,
        //         h: (180 - 2 * border as i32) as f32,
        //     },
        //     translation_matrix: glm::Mat4::new_translation(&glm::vec3(0.0, 0.0, -0.2f32)),
        // });
        // let mut ball = self.world.add_entity();
        // ball.assign(Ball {
        //     r: 2,
        //     spawned_a_new: 1884,
        // });
        // ball.assign(Mover::new(1.0, 1.0));
        // ball.assign(Position::new(
        //     GAME_PIXEL_WIDTH as i32 / 2,
        //     GAME_PIXEL_HEIGHT as i32 / 2,
        // ));
        // // ball.assign(Gravity { value: 0f32 });
        // ball.assign(Collider::new(ColliderType::Rect {
        //     rect: RectF::with_size(2f32, 2f32),
        // }));

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
        self.scene.init();
        let world = &mut self.scene.world;
        Player::add_to_world(world);
    }

    fn update(&mut self) -> bool {
        self.scene.update();

        let xdiff;
        let ydiff;
        {
            let player = self.scene.world.find_first::<Controller>().unwrap();
            let player_position = player.get_component::<Position>().unwrap();
            xdiff = sign(player_position.x as f32 / (GAME_PIXEL_WIDTH as f32));
            ydiff = sign(player_position.y as f32 / (GAME_PIXEL_HEIGHT as f32));
        }
        if xdiff != 0 || ydiff != 0 {
            // We are in a different room.
            self.current_room = (
                (self.current_room.0 as i32 + xdiff) as usize,
                (self.current_room.1 as i32 + ydiff) as usize,
            );
            let new_level = self
                .map
                .get(self.current_room.0, self.current_room.1)
                .unwrap();
            let mut new_level = GameScene::with_map(new_level.to_owned());
            Player::move_from(&mut self.scene.world, &mut new_level.world);
            self.scene = new_level;
            self.scene.init();

            // Re position player
            let player = self.scene.world.find_first::<Controller>().unwrap();
            let mut player_position = player.get_component::<Position>().unwrap();
            let player_size = player.get_component::<Controller>().unwrap();
            match xdiff {
                x if x > 0 => {
                    player_position.x = 0;
                }
                x if x < 0 => {
                    player_position.x = GAME_PIXEL_WIDTH as i32 - player_size.width as i32;
                }
                _ => {}
            }

            match ydiff {
                y if y < 0 => {
                    player_position.y = GAME_PIXEL_HEIGHT as i32 - player_size.height as i32;
                }
                y if y > 0 => {
                    player_position.y = 0;
                }
                _ => {}
            }
        }
        return true;
    }

    fn render(&self, batch: &mut Batch) {
        {
            // Render into low-res target
            self.target.clear((0f32, 0.1f32, 0.2f32));
            self.scene.render(batch);

            // self.world.render(batch);
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
        // self.world.debug(imgui);
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
    let game = Foo::new();
    engine::run(game);
}

fn sign(x: f32) -> i32 {
    match x {
        x if x < 0.0 => -1,
        x if x <= 1.0 => 0,
        _ => 1,
    }
}
