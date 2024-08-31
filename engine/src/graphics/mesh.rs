extern crate gl;

use super::common::*;

#[derive(Debug)]
pub struct Mesh {
    id: u32,
    count: usize,
    vertex_buffer: u32,
    index_buffer: u32,
}

impl Mesh {
    pub fn new() -> Self {
        println!("expensive call! Creating new mesh");
        let mut vao = 0;
        let mut buffers: [u32; 2] = [0, 0];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(2, (&mut buffers) as *mut u32);
            gl::BindVertexArray(vao);
            // bind ARRAY_BUFFER to VAO
            {
                let stride =
                    (8 * core::mem::size_of::<f32>() + 4 * core::mem::size_of::<u8>()) as i32;
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[0]);
                // aPos;
                gl::VertexAttribPointer(
                    0,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
                );
                gl::EnableVertexAttribArray(0);
                // aColor
                gl::VertexAttribPointer(
                    1,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (5 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
                );
                gl::EnableVertexAttribArray(1);
                // aTexCoord
                gl::VertexAttribPointer(
                    2,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (0 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
                );
                gl::EnableVertexAttribArray(2);
                // typ (mult wash fill)
                gl::VertexAttribPointer(
                    3,
                    4,
                    gl::UNSIGNED_BYTE,
                    gl::TRUE,
                    stride,
                    (8 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
                );
                gl::EnableVertexAttribArray(3);
            }
            // bind EBO to VAO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[1]);
            while gl::GetError() != gl::NO_ERROR {
                panic!("Something went wrong creating Mesh")
            }
        }
        Mesh {
            id: vao,
            count: 0,
            vertex_buffer: buffers[0],
            index_buffer: buffers[1],
        }
    }

    pub fn set_data(&mut self, vertices: &[Vertex]) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const std::os::raw::c_void,
                // gl::STATIC_DRAW,
                gl::DYNAMIC_DRAW,
            );
            gl::BindVertexArray(0);
            while gl::GetError() != gl::NO_ERROR {
                panic!("OpenGL error Mesh#set_data")
            }
        }
        self.count = vertices.len();
    }

    pub fn set_index_data(&mut self, indices: &[u32]) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const std::os::raw::c_void,
                gl::STATIC_DRAW,
            );
            while gl::GetError() != gl::NO_ERROR {
                panic!("OpenGL error Mesh#set_index_data")
            }
        }
    }

    pub fn bind(&self) {
        if self.id == 0 {
            panic!("Trying to bind a non existing Mesh (vao == 0)");
        }
        unsafe {
            gl::BindVertexArray(self.id);
            while gl::GetError() != gl::NO_ERROR {
                panic!("OpenGL error Mesh#bind")
            }
        }
    }
}
