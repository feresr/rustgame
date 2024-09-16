mod aseprite;
mod components;
mod content;
mod scene;
mod system;

extern crate engine;
extern crate nalgebra_glm as glm;

use components::position::Position;
use content::content;
use engine::{
    ecs::World,
    graphics::{
        batch::*,
        blend::{self},
        common::*,
        target::*,
        texture::*,
    },
    Config, Game,
};
use imgui::Ui;
use scene::Scene;
use std::env;
use system::{
    animation_system::AnimationSystem, light_system::LightSystem, movement_system::MovementSystem,
    player_system::PlayerSystem, render_system::RenderSystem, room_system::RoomSystem,
};

const SCREEN_WIDTH: usize = GAME_PIXEL_WIDTH * 4;
const SCREEN_HEIGHT: usize = GAME_PIXEL_HEIGHT * 4;

const TILE_SIZE: usize = 8;
const GAME_PIXEL_WIDTH: usize = 320;
const GAME_PIXEL_HEIGHT: usize = 184;

const GAME_TILE_WIDTH: usize = GAME_PIXEL_WIDTH / TILE_SIZE;
const GAME_TILE_HEIGHT: usize = GAME_PIXEL_HEIGHT / TILE_SIZE;

struct Foo {
    gbuffer: Target,
    world: World,
    movement_system: MovementSystem,
    render_system: RenderSystem,
    player_system: PlayerSystem,
    room_system: RoomSystem,
    animation_system: AnimationSystem,
    light_system: Option<LightSystem>,
    screen_target: Target,
    screen_ortho: glm::Mat4,
    screen_rect: RectF,
}
impl Foo {
    fn new() -> Self {
        let screen_ortho = glm::ortho(
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            0f32,
            0.0f32,
            2f32,
        );
        Self {
            screen_ortho,
            gbuffer: Target::empty(),
            world: World::new(),
            movement_system: MovementSystem,
            render_system: RenderSystem,
            player_system: PlayerSystem,
            room_system: RoomSystem::new(),
            animation_system: AnimationSystem,
            light_system: None,
            screen_target: Target::screen(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
            screen_rect: RectF::with_size(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
        }
    }
}

impl Game for Foo {
    fn init(&mut self) {
        let attachments = [
            // Albedo
            TextureFormat::RGBA,
            // Shadows
            TextureFormat::RGBA,
            // Depth
            TextureFormat::DepthStencil,
        ];
        self.gbuffer = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &attachments,
        );
        self.player_system.init(&mut self.world);
        self.room_system.scene.init(&mut self.world);
        self.light_system = Some(LightSystem::new());
        // engine::audio().play_music(&content().tracks["music-1"]);
    }

    fn update(&mut self) -> bool {
        self.player_system.update(&mut self.world);
        self.movement_system.update(&mut self.world);
        self.room_system.update(&mut self.world);
        return true;
    }

    fn render(&self, batch: &mut Batch) {
        {
            // Render into low-res target (gbuffer)
            self.gbuffer.clear((0.1f32, 0.1f32, 0.24f32, 1.0f32));
            batch.set_sampler(&TextureSampler::nearest());
            self.animation_system.tick(&self.world);

            //
            batch.set_blend(blend::NORMAL);
            self.render_system.render(&self.world, batch);
            batch.render(&self.gbuffer, &self.room_system.camera);
            batch.clear();

            self.light_system
                .as_ref()
                .unwrap()
                .render(&self.world, batch);
            batch.clear();
            batch.set_blend(blend::ADDITIVE);
            batch.tex(
                &RectF {
                    x: 0f32,
                    y: 0f32,
                    w: GAME_PIXEL_WIDTH as f32,
                    h: GAME_PIXEL_HEIGHT as f32,
                },
                &self.light_system.as_ref().unwrap().color(),
                (1f32, 1f32, 1f32, 1.0f32),
            );
            batch.render(
                &self.gbuffer,
                &glm::ortho(
                    0f32,
                    GAME_PIXEL_WIDTH as f32,
                    0f32,
                    GAME_PIXEL_HEIGHT as f32,
                    0.0f32,
                    0.2f32,
                ),
            );

            batch.clear();
        }
        {
            // Render low-res target onto the screen
            batch.set_sampler(&TextureSampler::nearest());
            batch.tex(
                &self.screen_rect,
                &self.gbuffer.color(),
                (1.0f32, 1.0f32, 1.0f32, 1f32),
            );
            batch.render(&self.screen_target, &self.screen_ortho);
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
    let game = Foo::new();
    engine::run(game);
}
