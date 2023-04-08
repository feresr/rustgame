extern crate engine;

extern crate nalgebra_glm as glm;
use bevy_ecs::prelude::*;
use engine::{
    graphics::{batch::*, common::*, material::*, shader::*, target::*, texture::*},
    Slider,
};

#[derive(Component)]
struct Circle {}
#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    r: f32,
}
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
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
            uniform float time;
            void main()\n
            {\n
                vec2 c = (2.0 * gl_FragCoord.xy - u_resolution.xy) / u_resolution.x; 
                c = c * 25.0;
                if (length(sin(c + vec2(time,time))) < 0.20) {
                    FragColor = vec4(0.2);
                } else {
                    FragColor = vec4(0.05);
                }
            }";

#[derive(Resource)]
struct Assets {
    textures: Vec<Texture>,
}

fn main() {
    engine::start(&|world, update, render| {
        world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                r: 1.2,
            },
            Velocity {
                x: 0.0001,
                y: 0.00005,
            },
            Circle {},
        ));

        world.spawn(Slider { a: 0.0, b: 0.0, perspective: false});

        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        let mut material = Material::new(shader);
        let mut sampler = TextureSampler::default();
        sampler.filter = TextureFilter::Linear;

        let mut path = std::env::current_exe().unwrap();
        path.pop(); // removes executable name from path
        let path = path.display().to_string();
        let texture = Texture::from_path(String::from(path.clone() + "/brick.png").as_str());
        let texture2 = Texture::from_path(String::from(path + "/coin.png").as_str());
        let assets = Assets {
            textures: vec![texture, texture2],
        };

        material.set_texture("u_mytex", &assets.textures[0]);
        material.set_sampler("u_mytex", &sampler);

        world.insert_resource(assets);
        world.insert_resource(material);
        world.insert_resource(sampler);
        world.insert_non_send_resource(Target::new(200, 100, &[TextureFormat::RGBA]));

        update.add_system(updating);
        render.add_system(rendering);
    });
}

// This system moves each entity with a Position and Velocity component
// NonSend means (use main thread)
fn rendering(
    mut batch: NonSendMut<'_, Batch>,
    mut query: Query<'_, '_, (&mut Position, &Circle)>,
    mut qslider: Query<'_, '_, &mut Slider>,
    mut material: ResMut<'_, Material>,
    mut sampler: Res<'_, TextureSampler>,
    assets: Res<'_, Assets>,
) {
    batch.set_sampler(&sampler);
    let mut ortho_bottom = 0.0;
    let mut ortho_top = 0.0;
    for slider in &mut qslider {
        ortho_top = slider.a;
        ortho_bottom = slider.b;
    }
    for (position, _) in &mut query {
        //     // todo: this material unifomr is overwritten (since the material is shared)
        material.set_valuef("time", position.r);
        let rect1 = RectF {
            x: -5.0,
            y: -5.0,
            w: 10.0,
            h: 10.0,
        };
        batch.push_material(&material);
        batch.tex(&rect1, &assets.textures[1]);
        batch.pop_material();

        let mat = glm::Mat4::identity();
        let rot = glm::Mat4::from_scaled_axis(&glm::vec3(1.2, 1.0, 0.0) * position.r);
        let mat = rot * mat;
        batch.push_matrix(mat);
        batch.cube((0.0, 0.0), ortho_bottom);
        // batch.circle((0.0, 0.0), 1.0, 32);
        batch.pop_matrix();
    }

    let ratio = (SCREEN.width as f32) / SCREEN.height as f32;

    let width = 5.0;
    let height = (width / ratio) / 2.0;

    if qslider.iter().next().unwrap().perspective {
        let perspective = glm::perspective(ratio, 0.50, 0.02, 10.0);
        let perspective = perspective.prepend_translation(&glm::vec3(0.0, 0.0, -5.0));
        batch.render(&SCREEN, &perspective);
    } else {
        let ortho: glm::Mat4 = glm::ortho(-width / 2.0, width / 2.0, -height, height, -5.0, 5.0);
        batch.render(&SCREEN, &ortho);
    }
    batch.clear();
}

fn updating(mut query: Query<'_, '_, (&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.r += 0.01 + velocity.x;
        position.x += velocity.x;
        position.y += velocity.y;
    }
}
