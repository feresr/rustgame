pub mod batch;
pub mod common;
mod drawcall;
pub mod material;
pub mod mesh;
pub mod shader;
pub mod texture;
pub mod target;

pub const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec2 aPos;\n
            layout (location = 1) in vec2 aTexCoord;\n
            out vec2 TexCoord;
            void main()\n
            {\n
               gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);\n
               TexCoord = aTexCoord;
            }";
pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            out vec4 FragColor;\n
            uniform sampler2D u_texture;\n
            uniform vec3 a_color;\n
            void main()\n
            {\n
                FragColor = vec4(a_color, 1.0) + texture(u_texture, TexCoord);\n
            }";
