extern crate engine;

use engine::graphics::{batch::*, common::*, material::*, shader::*};

struct MyGame {
    position: (f32, f32),
    velocity: f32,
    // this could be a "content manager" instead
    material: Option<Material>,
    material2: Option<Material>,
    shader: Option<Shader>,
    test : bool
}

const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec2 aPos;\n
            void main()\n
            {\n
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);\n
            }";
const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            out vec4 FragColor;\n
            uniform vec3 color;
            void main()\n
            {\n
                FragColor = vec4(color.rgb, 1.0f);\n
            }";
const FRAGMENT_SHADER_SOURCE_2: &str = "#version 330 core\n
            out vec4 FragColor;\n
            uniform vec3 color;
            void main()\n
            {\n
                FragColor = vec4(color.rgb, 1.0f);\n
            }";

impl engine::Game for MyGame {
    fn render<'b>(&'b mut self, batch: &mut Batch<'b>) {
        let rect = RectF {
            x: -0.9,
            y: -0.9,
            w: 0.6,
            h: 0.6,
        };

        batch.push_material(self.material.as_ref().unwrap());

        if self.test == true {
            batch.tri((0.4, 0.2), (0.9, 0.2), (0.4, 0.9));
        }

        batch.circle(
            (-1.0 + batch.ui.io().mouse_pos[0]/400.0, 1.0 - batch.ui.io().mouse_pos[1]/ 300.0),
            0.2,
            3 + (self.position.0.abs() * 18.0) as u32,
        );

        batch.rect(&rect);

        batch.pop_material();

        if self.test == true {
            batch.tri((-0.5, 0.3), (-0.2, 0.2), (-0.4, 0.9));
        }
        batch.ui.checkbox("test", &mut self.test);
        batch.render();
    }

    fn init(&mut self) {
        self.shader = Option::Some(Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE));
        let material = Material::new(self.shader.clone().unwrap());
        self.material = Option::Some(material);

        self.shader = Option::Some(Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2));
        let material = Material::new(self.shader.clone().unwrap());
        self.material2 = Option::Some(material);
    }

    fn update(&mut self) {
        self.position.0 = f32::sin(self.velocity * 1.0) * 0.9;
        self.position.1 = f32::cos(self.velocity * 5.0) * 0.25;
        self.velocity += 0.008;

        let r = (1.0 + f32::sin(self.velocity * 2.0)) / 2.0;
        let g = (1.0 + f32::sin(self.velocity)) / 2.0;
        let b = (1.0 + f32::sin(self.velocity * 1.5)) / 2.0;
        self.material
            .as_mut()
            .unwrap()
            .set_value3f("color", (r, g, b));
        self.material2
            .as_mut()
            .unwrap()
            .set_value3f("color", (g, b, r));
    }
}

fn main() {
    engine::start(MyGame {
        position: (0.0, 0.0),
        velocity: 0.0,
        material: Option::None,
        material2: Option::None,
        shader: Option::None,
        test : true
    });
}
