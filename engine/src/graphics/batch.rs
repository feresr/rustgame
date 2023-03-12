extern crate gl;

use super::common::*;
use super::mesh::*;
use super::shader::*;

// Sprite batcher used to draw text and textures

pub struct Batch {
    mesh: Mesh,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    batch: DrawBatch,
}

struct DrawBatch {
    elements: i32,
}

impl Batch {
    pub fn init(&self) {
        Shader::new(super::VERTEX_SHADER_SOURCE, super::FRAGMENT_SHADER_SOURCE);
    }
    pub fn render(&mut self) {
        self.mesh.set_data(&self.vertices);
        self.mesh.set_index_data(&self.indices);
        self.mesh.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const std::os::raw::c_void,
            );
        }
    }
    pub fn rect(&mut self, rect: RectF, color: &Color) {

        self.indices.push(0);
        self.indices.push(1);
        self.indices.push(3);
        self.indices.push(1);
        self.indices.push(2);
        self.indices.push(3);

        self.vertices.push(Vertex {
            pos: (rect.x + rect.w, rect.y), // top right
        });
        self.vertices.push(Vertex {
            pos: (rect.x + rect.w, rect.y + rect.h), // bottom rigth
        });
        self.vertices.push(Vertex {
            pos: (rect.x, rect.y), // bottom left
        });
        self.vertices.push(Vertex {
            pos: (rect.x, rect.y + rect.h), // top left
        });
    }
    pub fn clear(&mut self) {
        self.batch.elements = 0;
        self.vertices.clear();
    }
}

impl Default for Batch {
    fn default() -> Self {
        Batch {
            mesh: Mesh::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            batch: DrawBatch { elements: 0 },
        }
    }
}
