pub mod batch;
pub mod common;
pub mod shader;
pub mod material;
pub mod mesh;
mod drawcall;

pub const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec2 aPos;\n
            void main()\n
            {\n
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);\n
            }";
pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            out vec4 FragColor;\n
            uniform float hello;\n
            void main()\n
            {\n
                FragColor = vec4(hello, 1.0f, 1.0f, 1.0f);\n
            }";
