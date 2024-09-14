pub mod batch;
pub mod common;
mod drawcall;
pub mod material;
pub mod mesh;
pub mod shader;
pub mod target;
pub mod texture;
pub mod blend;

pub const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec3 aPos;\n
            layout (location = 1) in vec4 aColor;\n
            layout (location = 2) in vec2 aTexCoord;\n
            layout (location = 3) in vec4 aType;\n
            uniform mat4 u_matrix;\n
            out vec2 TexCoord;\n
            out vec4 a_color;\n
            out vec4 a_type;\n
            
            void main()\n
            {\n
               gl_Position = u_matrix * vec4(aPos, 1.0);\n
               TexCoord = aTexCoord; \n
               a_color = aColor; \n
               a_type = aType; \n
            }";

// todo a_color should be a vec4
// todo a_type is (mult wash fill pad) document better
pub const FRAGMENT_SHADER_SOURCE: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            in vec4 a_color;\n
            in vec4 a_type;\n 
            layout(location = 0) out vec4 FragColor;\n

            uniform sampler2D u_texture;\n
            uniform ivec2 u_resolution;\n

            void main()\n
            {\n
                vec4 tex = texture(u_texture, TexCoord); \n
                FragColor = \n
                    a_type.x * tex * a_color + \n
                    a_type.y * tex.a * a_color + \n
                    a_type.z * a_color; \n 
            }";
