use common::{Debug, Keyboard};
use engine::{
    ecs::World,
    graphics::{
        self, batch::Batch, blend, common::RectF, material::Material, texture::TextureSampler,
    },
};

use crate::{
    components::{button::Button, light::LightSwitch},
    content::Content,
    scene::Scene,
    system::{
        animation_system::AnimationSystem, editor::Editor, light_system::LightSystem,
        movement_system::MovementSystem, player_system::PlayerSystem, render_system::RenderSystem,
        scene_system::SceneSystem,
    },
    target_manager::TargetManager,
};

pub const ROOM_COUNT_W: usize = 4;
pub const ROOM_COUNT_H: usize = 4;
pub const SCREEN_WIDTH: usize = GAME_PIXEL_WIDTH * 4;
pub const SCREEN_HEIGHT: usize = GAME_PIXEL_HEIGHT * 4;

pub const TILE_SIZE: usize = 8;
pub const GAME_PIXEL_WIDTH: usize = 320;
pub const GAME_PIXEL_HEIGHT: usize = 184;

pub const GAME_TILE_WIDTH: usize = GAME_PIXEL_WIDTH / TILE_SIZE;
pub const GAME_TILE_HEIGHT: usize = GAME_PIXEL_HEIGHT / TILE_SIZE;

pub const CRT_FRAGMENT_SOURCE: &str = include_str!("crt_shader.fs");

#[repr(C)]
pub struct GameState {
    world: World,
    batch: Batch,
    movement_system: MovementSystem,
    render_system: RenderSystem,
    player_system: PlayerSystem,
    pub scene_system: SceneSystem,
    light_system: LightSystem,
    screen_rect: RectF,
    post_processing_material: Material,
    show_editor: bool,
    editor: Editor,
    pub target_manager: TargetManager, // low-res target
}

impl GameState {
    pub fn init_systems(&mut self) {
        self.player_system.init(&mut self.world);
        self.scene_system.scene.init(&mut self.world);
    }

    pub fn new() -> Self {
        let world = World::new();
        let player_system = PlayerSystem;

        let scene_system = SceneSystem::new();

        let light_system = LightSystem::new();
        let render_system = RenderSystem::new();

        let target_manager = TargetManager::new();

        let crt_shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, CRT_FRAGMENT_SOURCE);
        let mut post_processing_material =
            Material::with_sampler(crt_shader, TextureSampler::nearest());
        let sampler = TextureSampler::nearest();
        // post_processing_material.set_sampler("u_color_texture", &sampler);
        // The render system gives a albedo * normal mult color texture (which takes into consideration light)
        // post_processing_material.set_texture("u_color_texture", target_manager.color.color());
        post_processing_material.set_sampler("u_light_texture", &sampler);
        // The light system gives a black and white stencil for drawing the light cirlces (and hard shadows)
        post_processing_material.set_texture("u_light_texture", target_manager.lights.color());
        // engine::audio().play_music(&content().tracks["music-1"]);

        let mut batch = graphics::batch::Batch::default();

        // Render all maps colors and normals into a huge texture
        Content::map().prerender(
            &mut batch,
            &target_manager.maps_color,
            &target_manager.maps_normal,
            &target_manager.maps_outline,
        );

        Self {
            world,
            batch,
            movement_system: MovementSystem,
            render_system,
            player_system,
            scene_system,
            light_system,
            screen_rect: RectF::with_size(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            post_processing_material,
            show_editor: false,
            editor: Editor::default(),
            target_manager,
        }
    }

    // This is so that we can see shader updates when re-loading the game lib
    pub fn refresh(&mut self) {
        Content::map().prerender(
            &mut self.batch,
            &self.target_manager.maps_color,
            &self.target_manager.maps_normal,
            &self.target_manager.maps_outline,
        );
        self.target_manager.screen.clear((0f32, 0f32, 0f32, 0f32));
        let crt_shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, CRT_FRAGMENT_SOURCE);
        let mut post_processing_material =
            Material::with_sampler(crt_shader, TextureSampler::nearest());
        let sampler = TextureSampler::nearest();
        // post_processing_material.set_sampler("u_texture", &sampler);
        // The render system gives a albedo * normal mult color texture (which takes into consideration light)
        // post_processing_material.set_texture("u_texture", self.target_manager.color.color());
        post_processing_material.set_sampler("u_light_texture", &sampler);
        // The light system gives a black and white stencil for drawing the light cirlces (and hard shadows)
        post_processing_material.set_texture("u_light_texture", self.target_manager.lights.color());
        self.post_processing_material = post_processing_material;
        self.batch.clear();
    }

    pub fn update(&mut self) -> bool {
        if Keyboard::pressed(engine::Keycode::Tab) {
            dbg!("show editor pressed");
            self.show_editor = !self.show_editor;
        }
        Debug::window("Game");
        Debug::display(&"Press tab to toggle editor");
        Debug::button(|| {
            // self.show_editor == !self.show_editor;
            dbg!("shit");
        });
        Debug::separator();
        Debug::display(&format!("Showing editor: {} ", self.show_editor));

        if !self.show_editor {
            // Make sure we are in the right screen
            self.scene_system.update(&mut self.world);
            // Control / update player
            self.player_system.update(&mut self.world);
            // Actually move stuff
            self.movement_system.update(&mut self.world);
            Button::update(&mut self.world);
            LightSwitch::update(&mut self.world);
        } else {
            self.editor.update();
        }
        true
    }

    pub fn render(&mut self) {
        engine::update();

        // Render into low-res target
        {
            self.target_manager
                .game
                .clear((0.1f32, 0.1f32, 0.24f32, 1.0f32));
            self.batch.set_sampler(&TextureSampler::nearest());
            AnimationSystem::tick(&self.world); // ??

            self.batch.set_blend(blend::NORMAL);
            self.render_system
                .render(&self.world, &mut self.batch, &self.target_manager.color);
            self.batch.clear();

            self.light_system.render(
                &self.world,
                &mut self.batch,
                &mut self.target_manager.lights,
            );

            self.batch.clear();

            // Render the 'color' + 'lighting' into the final 'game' frame target
            self.batch.push_material(&self.post_processing_material);
            self.batch.tex(
                &RectF::with_size(GAME_PIXEL_WIDTH as f32, GAME_PIXEL_HEIGHT as f32),
                self.target_manager.color.color(),
                (1.0f32, 1.0f32, 1.0f32, 1f32),
            );
            self.batch.render(&self.target_manager.game);
            self.batch.pop_material();

            self.batch.clear();
        }

        // Finally, render low-res target onto the screen
        {
            self.batch.set_sampler(&TextureSampler::nearest());
            if self.show_editor {
                self.editor.render(&mut self.batch, &self.target_manager);
            } else {
                self.batch.tex(
                    &self.screen_rect,
                    self.target_manager.game.color(),
                    (1.0f32, 1.0f32, 1.0f32, 1f32),
                );
            }

            self.target_manager.screen.clear((0f32, 0f32, 0f32, 1f32));
            self.batch.render(&self.target_manager.screen);
        }
    }
}
