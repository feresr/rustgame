extern crate gl;

use std::f32::consts::TAU;

use super::common::*;
use super::drawcall;
use super::material::*;
use super::mesh::*;

// Sprite batcher used to draw text and textures
pub struct Batch<'a> {
    mesh: &'a mut Mesh,
    vertices: &'a mut Vec<Vertex>,
    indices: &'a mut Vec<u32>,
    batches: Vec<DrawBatch>,
    material_stack: Vec<Material>,
    default_material: &'a Material,
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
        self.mesh.bind();
        self.mesh.set_data(&self.vertices);
        self.mesh.set_index_data(&self.indices);
        println!("material stack is {:?}", self.material_stack.len());

        for batch in self.batches.iter() {
            let mut pass = drawcall::DrawCall::new(&self.mesh, &batch.material);
            pass.index_start = batch.offset * 3;
            pass.index_count = batch.elements * 3;
            if pass.index_count == 0 {
                println!("pass empty {}", pass.index_count);
                continue;
            }
            pass.perform();
        }
    }

    pub fn rect(&mut self, rect: &RectF) {
        let last_vertex_index = self.indices.len() as u32;
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

        self.current_batch().elements += 2;
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
        let last_vertex_index = self.indices.len() as u32;
        self.indices.push(0 + last_vertex_index);
        self.indices.push(1 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);

        self.vertices.push(Vertex { pos: pos0 });
        self.vertices.push(Vertex { pos: pos1 });
        self.vertices.push(Vertex { pos: pos2 });
        self.current_batch().elements += 1;
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn push_material(&mut self, material: &Material) {
        let current: &mut DrawBatch = self.current_batch();
        let m = current.material.clone();
        self.material_stack.push(m);
        print!("material stack increased to {}", self.material_stack.len());

        let current: &mut DrawBatch = self.current_batch();

        // material already applied, return
        if current.material == *material {
            println!("Material already applied!");
            return;
        };

        // current batch is empty, replace material
        if current.elements == 0 {
            println!("swapping existing mat material");
            current.material = material.clone();
            return;
        }

        // create new batch
        let value = DrawBatch {
            offset: current.offset + current.elements,
            elements: 0,
            material: material.clone(),
        };
        self.batches.push(value);
    }

    pub fn pop_material(&mut self) {
        let mat = if let Some(previus_material) = self.material_stack.pop() {
            previus_material
        } else {
            self.default_material.clone()
        };
        print!("material stack decreased to {}", self.material_stack.len());
        let current = self.current_batch();
        let value = DrawBatch {
            offset: current.offset + current.elements,
            elements: 0,
            material: mat,
        };
        self.batches.push(value);
    }

    pub fn new(
        mesh: &'a mut Mesh,
        material: &'a Material,
        vertices: &'a mut Vec<Vertex>,
        indices: &'a mut Vec<u32>,
    ) -> Batch<'a> {
        return Batch {
            mesh,
            vertices,
            indices,
            material_stack: Vec::new(),
            batches: Vec::new(),
            default_material: material,
        };
    }

    fn current_batch(&mut self) -> &mut DrawBatch {
        if self.batches.is_empty() {
            let value = DrawBatch {
                offset: 0,
                elements: 0,
                material: self.default_material.clone(),
            };
            self.batches.push(value);
        }
        return self.batches.last_mut().unwrap();
    }
}

pub struct DrawBatch {
    offset: i64,
    elements: i64,
    material: Material,
}
