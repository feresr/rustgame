extern crate engine;

use engine::graphics::{batch::*, common::*, material::*, shader::*};

struct MyGame {
    position: (f32, f32),
    velocity: f32,
    // this could be a "content manager" instead
    material: Option<Material>,
    shader: Option<Shader>,
}

pub const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec2 aPos;\n
            void main()\n
            {\n
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);\n
            }";
pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            out vec4 FragColor;\n
            void main()\n
            {\n
                FragColor = vec4(0.0, 0.8f, 0.2f, 1.0f);\n
            }";

impl engine::Game for MyGame {
    fn render<'b>(&'b self, batch: &mut Batch<'b>) {
        let rect = RectF {
            x: -0.9,
            y: -0.9,
            w: 0.6,
            h: 0.6,
        };

        batch.tri((0.4, 0.2), (0.9, 0.2), (0.4, 0.9));

        batch.push_material(self.material.as_ref().unwrap());

        batch.circle(
            self.position,
            0.2,
            3 + (self.position.0.abs() * 10.0) as u32,
        );

        batch.pop_material();

        batch.rect(&rect);

        batch.render();
    }

    fn init(&mut self) {
        // open gl is not active yet
        self.shader = Option::Some(Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE));
        let material = Material::new(self.shader.clone().unwrap());
        self.material = Option::Some(material);
    }

    fn update(&mut self) {
        self.position.0 = f32::sin(self.velocity * 1.0) * 0.9;
        self.position.1 = f32::cos(self.velocity * 5.0) * 0.25;
        self.velocity += 0.008;
    }
}

fn main() {
    engine::start(&mut MyGame {
        position: (0.0, 0.0),
        velocity: 0.0,
        material: Option::None,
        shader: Option::None,
    });
}
