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
    scene::Scene,
    system::{
        animation_system::AnimationSystem, light_system::LightSystem,
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

pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            in vec4 a_color;\n
            in vec4 a_type;\n 
            layout(location = 0) out vec4 FragColor;\n

            uniform sampler2D u_color_texture;\n
            uniform sampler2D u_light_texture;\n

            uniform float u_light_radius;\n

            uniform ivec2 u_resolution;\n

            void main()\n
            {\n
                vec4 color = texture(u_color_texture, TexCoord); \n
                vec4 light = texture(u_light_texture, TexCoord); \n

                color = color + (light.x) * vec4(0.15); \n 
                // color = mix(color * vec4(0.60), color, 0.5); \n 

                float crtIntensity = 0.70; \n // 0 = max 1 = min
                float crt = (sin(gl_FragCoord.y * 3.14) + 1.0) * 0.5; \n
                crt = (crt * (1.0 - (crtIntensity))) + crtIntensity; \n
                // crt = (crt * 0.50) + 0.50; \n
                crt = mix(crt, 1.0, light.x); \n

                FragColor = vec4(color.rgb, 1.0) * vec4(crt, crt, crt, 1.0); \n

            }";

#[repr(C)]
pub struct GameState {
    gbuffer: Target,
    world: World,
    batch: Batch,
    movement_system: MovementSystem,
    render_system: RenderSystem,
    player_system: PlayerSystem,
    scene_system: SceneSystem,
    animation_system: AnimationSystem,
    light_system: LightSystem,
    screen_ortho: glm::Mat4,
    screen_rect: RectF,
    screen_target: Target,
    material: Material,
}

impl GameState {
    pub fn new() -> Self {
        let shader = graphics::shader::Shader::new(
            graphics::VERTEX_SHADER_SOURCE,
            graphics::FRAGMENT_SHADER_SOURCE,
        );
        let batch_default_material = graphics::material::Material::new(shader);
        let mesh = graphics::mesh::Mesh::new();
        let batch = graphics::batch::Batch::new(mesh, batch_default_material);
        let screen_ortho = glm::ortho(
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            0f32,
            0.0f32,
            2f32,
        );

        let attachments = [
            // Albedo
            TextureFormat::RGBA,
            // Shadows
            TextureFormat::RGBA,
            // Depth
            TextureFormat::DepthStencil,
        ];

        let gbuffer = Target::new(
            GAME_PIXEL_WIDTH as i32,
            GAME_PIXEL_HEIGHT as i32,
            &attachments,
        );

        let mut world = World::new();
        let player_system = PlayerSystem;
        player_system.init(&mut world);

        let mut scene_system = SceneSystem::new();
        scene_system.scene.init(&mut world);

        let light_system = LightSystem::new();
        let render_system = RenderSystem::new();

        let animation_system = AnimationSystem;

        let shader =
            graphics::shader::Shader::new(graphics::VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        let mut material = Material::with_sampler(shader, TextureSampler::nearest());
        let sampler = TextureSampler::nearest();
        material.set_sampler("u_color_texture", &sampler);
        material.set_texture("u_color_texture", &render_system.color());
        material.set_sampler("u_light_texture", &sampler);
        material.set_texture("u_light_texture", &light_system.color());
        // engine::audio().play_music(&content().tracks["music-1"]);

        Self {
            screen_ortho,
            gbuffer,
            world,
            batch,
            movement_system: MovementSystem,
            render_system,
            player_system,
            scene_system,
            animation_system,
            light_system,
            screen_target: Target::screen(SCREEN_WIDTH as i32 * 2, SCREEN_HEIGHT as i32 * 2),
            screen_rect: RectF::with_size(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            material,
        }
    }

    pub fn update(&mut self, keyboard: &Keyboard) -> bool {
        self.player_system.update(&mut self.world, keyboard);
        self.movement_system.update(&mut self.world);
        self.scene_system.update(&mut self.world);
        Button::update(&mut self.world);
        LightSwitch::update(&mut self.world);
        return true;
    }

    pub fn render(&mut self) {
        engine::update();
        {
            // Render into low-res target (gbuffer)
            self.gbuffer.clear((0.1f32, 0.1f32, 0.24f32, 1.0f32));
            self.batch.set_sampler(&TextureSampler::nearest());
            self.animation_system.tick(&self.world);

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
            self.batch.pop_material();

            self.batch.clear();
        }
        {
            // Render low-res target onto the screen
            self.batch.set_sampler(&TextureSampler::nearest());
            self.batch.tex(
                &self.screen_rect,
                &self.gbuffer.color(),
                (1.0f32, 1.0f32, 1.0f32, 1f32),
            );
            // TODO
            self.batch.render(&self.screen_target, &self.screen_ortho);
        }
    }

    #[allow(dead_code)]
    fn debug(&self, imgui: &Ui) {
        self.world.debug(imgui);
    }
}
