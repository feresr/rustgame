extern crate engine;

use std::cell::RefMut;

use bevy_ecs::prelude::*;
use engine::graphics::{batch::*, common::*, material::*, shader::*, target::*, texture::*};

#[derive(Component)]
struct Circle {}
#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec2 aPos;\n
            layout (location = 1) in vec2 aTexCoord;\n
            out vec2 TexCoord;
            void main()\n
            {\n
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);\n
               TexCoord = aTexCoord;
            }";
const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            out vec4 FragColor;\n
            uniform vec3 color;
            void main()\n
            {\n
                FragColor = vec4(color.rgb, 1.0f);\n
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

fn main() {
    engine::start(&|world, update, render| {
        world.spawn((
            Position { x: -2.0, y: 0.0 },
            Velocity { x: 0.001, y: 0.0 },
            Circle {},
        ));

        world.spawn((
            Position { x: 2.0, y: 0.2 },
            Velocity {
                x: -0.001,
                y: 0.0000,
            },
            Circle {},
        ));
        world.spawn((
            Position { x: -2.0, y: -0.9 },
            Velocity { x: 0.001, y: 0.000 },
            Circle {},
        ));

        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        let material = Material::new(shader);
        world.insert_resource(material);

        let mut path = std::env::current_exe().unwrap();
        path.pop(); // removes executable name from path
        let path = path.display().to_string();
        let texture = Texture::from_path(String::from(path + "/happy.jpg").as_str());
        world.insert_resource(texture);

        update.add_system(updating);
        render.add_system(rendering);
    });
}

// This system moves each entity with a Position and Velocity component
// NonSend means (use main thread)
fn rendering(
    mut batch: NonSendMut<'_, Batch>,
    mut query: Query<'_, '_, (&Position, &Circle)>,
    mut material: ResMut<'_, Material>,
    texture: Res<'_, Texture>,
) {
    let mut sampler = TextureSampler::default();
    sampler.filter = TextureFilter::Nearest;
    // if self.test == true {
    //     sampler.filter = TextureFilter::Linear;
    // }
    batch.set_sampler(&sampler);
    // batch.set_sampler(&sampler);
    // batch.tex(&rect1, &self.texture.as_ref().unwrap());
    // batch.tex(&rect2, &self.texture2.as_ref().unwrap());
    // maybe forbid this?  batch.peek_material().set_sampler(&sampler);
    // let pos = batch.ui.io().mouse_pos;

    // material.set_value2f("mouse", (700.0 / 1400.0, 1.0));
    material.set_texture("u_mytex", &texture);
    material.set_sampler("u_mytex", &sampler);

    batch.push_material(&material);
    for (position, _) in &mut query {
        let rect1 = RectF {
            x: position.x,
            y: position.y,
            w: 1.0,
            h: 1.0,
        };
        batch.tex(&rect1, &texture);
    }
    batch.pop_material();

    // let offscreen_buffer = self.target.as_ref().unwrap();
    // offscreen_buffer.clear();
    // batch.render(offscreen_buffer);
    // batch.clear();

    // batch.set_sampler(&sampler);
    // batch.tex(&rect1, &self.target.as_ref().unwrap().attachments[0]);
    // batch.circle((pos[0] / 1400.0, pos[1] / 800.0), 0.1, 32);
    batch.render(&SCREEN);
    batch.clear();
}

fn updating(mut query: Query<'_, '_, (&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}
