use common::Keyboard;
use engine::{
    ecs::World,
    graphics::{
        self,
        batch::Batch,
        blend,
        common::RectF,
        material::Material,
        target::Target,
        texture::{TextureFormat, TextureSampler},
    },
};
use imgui::Ui;

use crate::{
    components::{button::Button, light::LightSwitch},
    content::Content,
    scene::Scene,
    system::{
        animation_system::AnimationSystem, editor::Editor, light_system::LightSystem,
        movement_system::MovementSystem, player_system::PlayerSystem, render_system::RenderSystem,
        scene_system::SceneSystem,
    },
};

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
    low_res_target: Target, // low-res target
    world: World,
    batch: Batch,
    movement_system: MovementSystem,
    render_system: RenderSystem,
    player_system: PlayerSystem,
    pub scene_system: SceneSystem,
    light_system: LightSystem,
    screen_ortho: glm::Mat4,
    screen_rect: RectF,
    screen_target: Target,
    material: Material,
    show_editor: bool,
    editor: Editor,
    pub content: Content,
}

impl GameState {
    pub fn init_systems(&mut self) {
        self.player_system.init(&mut self.world);
        self.scene_system.scene.init(&mut self.world);
    }

    pub fn new(content: Content) -> Self {
        let screen_ortho = glm::ortho(
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            0f32,
            0.0f32,
            2f32,
        );

        let gbuffer = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &[TextureFormat::RGBA],
        );

        let mut world = World::new();
        let player_system = PlayerSystem;

        let mut scene_system = SceneSystem::new();

        let light_system = LightSystem::new();
        let render_system = RenderSystem::new();

        let crt_shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, CRT_FRAGMENT_SOURCE);
        let mut post_processing_material =
            Material::with_sampler(crt_shader, TextureSampler::nearest());
        let sampler = TextureSampler::nearest();
        post_processing_material.set_sampler("u_color_texture", &sampler);
        // The render system gives a albedo * normal mult color texture (which takes into consideration light)
        post_processing_material.set_texture("u_color_texture", render_system.color());
        post_processing_material.set_sampler("u_light_texture", &sampler);
        // The light system gives a black and white stencil for drawing the light cirlces (and hard shadows)
        post_processing_material.set_texture("u_light_texture", light_system.color());
        // engine::audio().play_music(&content().tracks["music-1"]);

        let batch = graphics::batch::Batch::default();
        Self {
            screen_ortho,
            low_res_target: gbuffer,
            world,
            batch,
            movement_system: MovementSystem,
            render_system,
            player_system,
            scene_system,
            light_system,
            screen_target: Target::screen(SCREEN_WIDTH as i32 * 2, SCREEN_HEIGHT as i32 * 2),
            screen_rect: RectF::with_size(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            material: post_processing_material,
            show_editor: false,
            editor: Editor::default(),
            content,
        }
    }

    // This is so that we can see shader updates when re-loading the game lib
    pub fn refresh(&mut self) {
        let crt_shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, CRT_FRAGMENT_SOURCE);
        let mut post_processing_material =
            Material::with_sampler(crt_shader, TextureSampler::nearest());
        let sampler = TextureSampler::nearest();
        post_processing_material.set_sampler("u_color_texture", &sampler);
        // The render system gives a albedo * normal mult color texture (which takes into consideration light)
        post_processing_material.set_texture("u_color_texture", self.render_system.color());
        post_processing_material.set_sampler("u_light_texture", &sampler);
        // The light system gives a black and white stencil for drawing the light cirlces (and hard shadows)
        post_processing_material.set_texture("u_light_texture", self.light_system.color());
        self.material = post_processing_material;
    }

    pub fn update(&mut self, keyboard: &Keyboard) -> bool {
        if keyboard.pressed.contains(&engine::Keycode::Tab) {
            self.show_editor = !self.show_editor;
        }

        if !self.show_editor {
            // Make sure we are in the right screen
            self.scene_system.update(&mut self.world, keyboard);
            // Control / update player
            self.player_system.update(&mut self.world, keyboard);
            // Actually move stuff
            self.movement_system.update(&mut self.world);
            Button::update(&mut self.world);
            LightSwitch::update(&mut self.world);
        }
        return true;
    }

    pub fn render(&mut self) {
        engine::update();
        // Render into low-res target
        {
            self.low_res_target.clear((0.1f32, 0.1f32, 0.24f32, 1.0f32));
            self.batch.set_sampler(&TextureSampler::nearest());
            AnimationSystem::tick(&self.world);

            self.batch.set_blend(blend::NORMAL);
            self.render_system.render(&self.world, &mut self.batch);
            self.batch.clear();

            self.light_system.render(&self.world, &mut self.batch);

            self.batch.clear();

            self.batch.push_material(&self.material);
            self.batch.rect(
                &RectF {
                    x: 0f32,
                    y: 0f32,
                    w: GAME_PIXEL_WIDTH as f32,
                    h: GAME_PIXEL_HEIGHT as f32,
                },
                (1f32, 1f32, 1f32, 1.0f32),
            );
            self.batch.render(
                &self.low_res_target,
                &glm::ortho(
                    0f32,
                    GAME_PIXEL_WIDTH as f32,
                    0f32,
                    GAME_PIXEL_HEIGHT as f32,
                    0.0f32,
                    0.2f32,
                ),
            );
            self.batch.pop_material();

            self.batch.clear();
        }

        // Render low-res target onto the screen
        {
            self.batch.set_sampler(&TextureSampler::nearest());
            self.batch.tex(
                &self.screen_rect,
                self.low_res_target.color(),
                (1.0f32, 1.0f32, 1.0f32, 1f32),
            );
            if self.show_editor {
                self.editor.render(&mut self.batch);
            }
            self.batch.render(&self.screen_target, &self.screen_ortho);
        }
    }

    #[allow(dead_code)]
    fn debug(&self, imgui: &Ui) {
        self.world.debug(imgui);
    }
}
