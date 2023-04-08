extern crate engine;

extern crate nalgebra_glm as glm;
use bevy_ecs::prelude::*;
use engine::graphics::{batch::*, common::*, material::*, shader::*, target::*, texture::*};

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
            out vec2 TexCoord;
            void main()\n
            {\n
               gl_Position = u_matrix * vec4(aPos, 1.0);\n
               TexCoord = aTexCoord;
            }";
const FRAGMENT_SHADER_SOURCE_2: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            out vec4 FragColor;\n
            uniform vec2 mouse;\n
            uniform sampler2D u_mytex;\n
            uniform sampler2D u_texture;\n
            void main()\n
            {\n
                vec4 a = texture(u_mytex, TexCoord);\n
                vec4 b = texture(u_texture, TexCoord);\n
                FragColor = mix(a, b, mouse.x);\n
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

        world.spawn((
            Position {
                x: 1.0,
                y: 0.2,
                r: 0.0,
            },
            Velocity {
                x: -0.0001,
                y: 0.0000,
            },
            Circle {},
        ));
        world.spawn((
            Position {
                x: -1.0,
                y: -0.9,
                r: 0.5,
            },
            Velocity { x: 0.00, y: 0.001 },
            Circle {},
        ));

        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        let mut material = Material::new(shader);
        let mut sampler = TextureSampler::default();
        sampler.filter = TextureFilter::Nearest;

        let mut path = std::env::current_exe().unwrap();
        path.pop(); // removes executable name from path
        let path = path.display().to_string();
        let texture = Texture::from_path(String::from(path.clone() + "/happy.jpg").as_str());
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
    mut material: ResMut<'_, Material>,
    mut sampler: Res<'_, TextureSampler>,
    assets: Res<'_, Assets>,
) {
    batch.set_sampler(&sampler);


    batch.push_material(&material);
    for (mut position, _) in &mut query {
        //     // todo: this material unifomr is overwritten (since the material is shared)
        material.set_value2f("mouse", (position.r.cos(), position.y));
        let rect1 = RectF {
            x: -1.0,
            y: -1.0,
            w: 2.0,
            h: 2.0,
        };
        let mat = glm::Mat4::identity();
        let rot = glm::Mat4::from_scaled_axis(&glm::vec3(1.2, 1.0, 0.6) * position.r);
        let mat = rot * mat;
        batch.push_matrix(mat);
        batch.tex(&rect1, &assets.textures[1]);
        batch.pop_matrix();
    }
    batch.pop_material();

    // batch.render(&SCREEN, &glm::ortho(0.0, 5.0, 0.0, 5.0, -2.0, 2.0));
    let perspective = glm::perspective(1.4, 0.75, 0.02, 10.0);
    let perspective = perspective.prepend_translation(&glm::vec3(0.0, 0.0, -5.0));
    batch.render(&SCREEN, &perspective);

    batch.clear();
}

fn updating(mut query: Query<'_, '_, (&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.r += 0.01 + velocity.x;
        position.x += velocity.x;
        position.y += velocity.y;
    }
}
