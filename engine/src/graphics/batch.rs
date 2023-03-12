extern crate gl;

use std::f32::consts::TAU;
use std::ops::Index;

use super::common::*;
use super::mesh::*;
use super::shader::*;

// Sprite batcher used to draw text and textures
pub struct Batch {
    mesh: Mesh,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
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

    pub fn rect(&mut self, rect: &RectF) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.push(1 + last_vertex_index);
        self.indices.push(0 + last_vertex_index);
        self.indices.push(3 + last_vertex_index);
        self.indices.push(0 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);
        self.indices.push(3 + last_vertex_index);

        // bottom right
        self.vertices.push(Vertex {
            pos: (rect.x + rect.w, rect.y),
        });
        // top rigth
        self.vertices.push(Vertex {
            pos: (rect.x + rect.w, rect.y + rect.h),
        });
        // bottom left
        self.vertices.push(Vertex {
            pos: (rect.x, rect.y),
        });
        // top left
        self.vertices.push(Vertex {
            pos: (rect.x, rect.y + rect.h),
        });
    }

    pub fn circle(&mut self, center: (f32, f32), radius: f32, steps: u32) {
        let mut last = (center.0 + radius, center.1);
        for i in 0..=steps {
            let radians = (i as f32 / steps as f32) * TAU;
            let next = (
                center.0 + f32::cos(radians) * radius,
                center.1 + f32::sin(radians) * radius,
            );
            self.tri(last, next, center);
            last = next;
        }
    }

    pub fn tri(&mut self, pos0: (f32, f32), pos1: (f32, f32), pos2: (f32, f32)) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.push(0 + last_vertex_index);
        self.indices.push(1 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);

        self.vertices.push(Vertex { pos: pos0 });
        self.vertices.push(Vertex { pos: pos1 });
        self.vertices.push(Vertex { pos: pos2 });
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

impl Default for Batch {
    fn default() -> Self {
        Batch {
            mesh: Mesh::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}
