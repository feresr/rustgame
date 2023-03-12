pub mod batch;
pub mod common;
mod shader;
mod mesh;

const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec2 aPos;\n
            void main()\n
            {\n
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);\n
            }";
const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            out vec4 FragColor;\n
            void main()\n
            {\n
                FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);\n
            }";
