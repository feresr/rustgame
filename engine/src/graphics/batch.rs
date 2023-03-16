extern crate gl;

use std::f32::consts::TAU;

use super::common::*;
use super::drawcall;
use super::material::*;
use super::mesh::*;
use super::shader::*;

// Sprite batcher used to draw text and textures
pub struct Batch<'a> {
    mesh: Mesh,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    batches: Vec<DrawBatch<'a>>,
    default_shader: &'a Shader,
}

impl<'a> Batch<'a> {
    pub fn init<'b>(&self) {}

    pub fn render(&mut self) {
        if self.batches.is_empty() {
            // nothing to draw
            println!("nothing to draw");
            return;
        }

        // upload data to gpu
        self.mesh.set_data(&self.vertices);
        self.mesh.set_index_data(&self.indices);
        self.mesh.bind();

        for batch in self.batches.iter() {
            let mut pass = drawcall::DrawCall::new(&self.mesh, &batch.material);
            pass.index_start = batch.offset * 3;
            pass.index_count = batch.elements * 3;
            pass.perform();
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

        self.last_batch().elements += 2;
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
        self.last_batch().elements += 1;
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn push_material(&mut self, _material: &Material<'_>) {}

    pub fn new(shader: &'a Shader) -> Self {
        let draw_batch = DrawBatch {
            offset: 0,
            elements: 0,
            material: Material::new(shader),
        };
        return Batch {
            mesh: Mesh::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            batches: vec![draw_batch],
            default_shader : shader,
        };
    }

    fn last_batch(&mut self) -> &mut DrawBatch<'a> {
        if self.batches.is_empty() {
            let value = DrawBatch {
                offset: 0,
                elements: 0,
                material: Material::new(self.default_shader),
            };
            self.batches.push(value);
        }
        return self.batches.last_mut().unwrap();
    }
}

pub struct DrawBatch<'a> {
    offset: i64,
    elements: i64,
    material: Material<'a>,
}