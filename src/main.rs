mod components;
mod content;
mod player;
mod scene;

extern crate engine;
extern crate nalgebra_glm as glm;

use components::{
    background::Background,
    collider::{Collider, ColliderType},
    controller::Controller,
    mover::Mover,
    position::Position,
    room::{Room, Tile},
};

use content::Content;
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
    screen_rect: RectF,
    gbuffer: Target,
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
            screen_rect: RectF::with_size(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            gbuffer: Target::empty(),
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
            GAME_PIXEL_HEIGHT as f32,
            0f32,
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

        let attachments = [TextureFormat::RGBA, TextureFormat::DepthStencil];
        self.gbuffer = Target::new(
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
            let player = self.scene.world.find_first::<Controller>().expect("No player found in world");
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
        let SCREEN = Target::screen(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
        {
            // Render into low-res target (gbuffer)
            self.gbuffer.clear((0f32, 0f32, 0.0f32, 1.0f32));
            batch.set_sampler(&TextureSampler::nearest());
            batch.circle((0f32, 0f32), 32f32, 9, (0f32, 1f32, 1f32));
            self.scene.render(batch);
            batch.circle((29f32, 40f32), 32f32, 9, (1f32, 1f32, 0f32));
            batch.render(&self.gbuffer, &self.ortho);
            batch.clear();
        }
        {
            // Render low-res target to screen
            batch.set_sampler(&TextureSampler::nearest());
            batch.tex(&self.screen_rect, &self.gbuffer.color(), (1f32, 1f32, 1f32));
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
