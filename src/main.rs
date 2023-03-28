extern crate engine;

use engine::graphics::{batch::*, common::*, material::*, shader::*, texture::*};

struct MyGame {
    position: (f32, f32),
    velocity: f32,
    // this could be a "content manager" instead
    material: Option<Material>,
    material2: Option<Material>,
    shader: Option<Shader>,
    test: bool,
    texture: Option<Texture>,
    texture2: Option<Texture>,
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
        let mut path = std::env::current_exe().unwrap();
        path.pop(); // remove exe name from path

        let rect1 = RectF {
            x: -0.9,
            y: -0.9,
            w: 0.6,
            h: 0.6,
        };

        let rect2 = RectF {
            x: 0.2,
            y: -0.9,
            w: 2.1,
            h: 2.1,
        };

        let mut sampler = TextureSampler::default();
        sampler.filter = TextureFilter::Nearest;
        if self.test == true {
            sampler.filter = TextureFilter::Linear;
        }
        batch.set_sampler(&sampler);
        batch.tex(&rect1, &self.texture.as_ref().unwrap());
        batch.tex(&rect2, &self.texture2.as_ref().unwrap());
        // maybe forbid this?  batch.peek_material().set_sampler(&sampler);

        let pos = batch.ui.io().mouse_pos;
        self.material2.as_mut().unwrap().set_value3f("color", (1.0, 0.0, 0.0));
        batch.push_material(&self.material2.as_ref().unwrap());
        batch.circle((pos[0] / 200.0, pos[1] / 100.0), 0.1, 32);
        batch.pop_material();
        batch.circle((pos[0] / 100.0, pos[1] / 50.0), 0.1, 32);

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
        let mut path = std::env::current_exe().unwrap();
        path.pop();

        let t = Texture::new(String::from(path.display().to_string() + "/coin.png").as_str());
        self.texture = Option::Some(t);

        let t = Texture::new(String::from(path.display().to_string() + "/happy.jpg").as_str());
        self.texture2 = Option::Some(t);
    }

    fn update(&mut self) {
        self.position.0 = f32::sin(self.velocity * 1.0) * 0.9;
        self.position.1 = f32::cos(self.velocity * 5.0) * 0.25;
        self.velocity += 0.008;

        let r = (1.0 + f32::sin(self.velocity * 2.0)) / 2.0;
        let g = (1.0 + f32::sin(self.velocity)) / 2.0;
        let b = (1.0 + f32::sin(self.velocity * 1.5)) / 2.0;
        // self.material
        //     .as_mut()
        //     .unwrap()
        //     .set_value3f("color", (r, g, b));
        // self.material2
        //     .as_mut()
        //     .unwrap()
        //     .set_value3f("color", (g, b, r));
    }
}

fn main() {
    engine::start(MyGame {
        position: (0.0, 0.0),
        velocity: 0.0,
        material: Option::None,
        material2: Option::None,
        shader: Option::None,
        test: true,
        texture: Option::None,
        texture2: Option::None,
    });
}
