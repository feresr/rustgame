extern crate gl;
use super::common::*;

pub struct Mesh {
    id: u32,
    count: usize,
    vertex_buffer: u32,
    index_buffer: u32,
}

impl Mesh {
    pub fn new() -> Self {
        let mut vao = 0;
        let mut buffers: [u32; 2] = [0, 0];
        unsafe {
            gl::GenVertexArrays(1, (&mut vao) as *mut u32);
            gl::GenBuffers(2, (&mut buffers) as *mut u32);
            gl::BindVertexArray(vao);
            // bind ARRAY_BUFFER to VAO
            {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[0]);
                gl::VertexAttribPointer(
                    0,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    (2 * core::mem::size_of::<f32>()) as i32,
                    std::ptr::null(),
                );
                gl::EnableVertexAttribArray(0);
            }
            // bind EBO to VAO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[1]);
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
                gl::STATIC_DRAW,
            );
            gl::BindVertexArray(0);
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
        }
    }

    pub fn bind(&self) {
        if self.id == 0 {
            panic!("Trying to bind a non existing Mesh (vao == 0)");
        }
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
}
