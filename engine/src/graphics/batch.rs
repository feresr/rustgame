extern crate gl;

use std::f32::consts::TAU;

use imgui::Ui;

use super::common::*;
use super::drawcall;
use super::material::*;
use super::mesh::*;
use super::target::Target;
use super::texture::*;

// Sprite batcher used to draw text and textures
pub struct Batch<'a> {
    mesh: &'a mut Mesh,
    vertices: &'a mut Vec<Vertex>,
    indices: &'a mut Vec<u32>,
    batches: Vec<DrawBatch>,
    material_stack: Vec<Material>,
    default_material: &'a Material,
    pub ui: &'a Ui,
}

impl<'a> Batch<'a> {
    pub fn init<'b>(&self) {}

    pub fn render(&mut self, target : &Target) {
        if self.batches.is_empty() {
            // nothing to draw
            println!("nothing to draw");
            return;
        }

        // upload data to gpu
        self.mesh.bind();
        self.mesh.set_data(&self.vertices);
        self.mesh.set_index_data(&self.indices);

        self.ui
            .window("Draw Batches")
            .size([800.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                for batch in self.batches.iter() {
                    if batch.elements == 0 {
                        continue;
                    }
                    self.ui.text(format!("elements: {}", batch.elements));
                    self.ui.text(format!("offset: {}", batch.offset));
                    self.ui.text(format!("texture: {}", batch.texture.id));
                    self.ui
                        .text(format!("sampler: {}", batch.texture_sampler.filter));
                    self.ui.separator();
                }

                self.ui.text(format!("vertices: {:?}", self.vertices));
                self.ui.separator();
                self.ui.text(format!("indices: {:?}", self.indices));
                self.ui.separator();
            });

        for batch in self.batches.iter_mut() {
            batch.material.set_texture("u_texture", &batch.texture);
            batch.material.set_sampler(&batch.texture_sampler);

            let mut pass = drawcall::DrawCall::new(&self.mesh, &batch.material, target);

            pass.index_start = batch.offset * 3;
            pass.index_count = batch.elements * 3;
            if pass.index_count == 0 {
                println!("pass empty {}", pass.index_count);
                continue;
            }
            pass.perform();
        }
    }

    pub fn set_sampler(&mut self, sampler: &TextureSampler) {
        let current = self.current_batch();
        if current.elements > 0 && *sampler != current.texture_sampler {
            self.push_batch();
        }
        let current = self.current_batch();
        current.texture_sampler = sampler.clone();
    }

    fn push_batch(&mut self) {
        let current = self.current_batch();
        let value = DrawBatch {
            offset: current.offset + current.elements,
            elements: 0,
            material: current.material.clone(),
            texture: current.texture,
            ..*current
        };
        self.batches.push(value);
    }

    fn push_quad(
        &mut self,
        pos0: (f32, f32),
        pos1: (f32, f32),
        pos2: (f32, f32),
        pos3: (f32, f32),
        tex0: (f32, f32),
        tex1: (f32, f32),
        tex2: (f32, f32),
        tex3: (f32, f32),
    ) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.push(1 + last_vertex_index);
        self.indices.push(0 + last_vertex_index);
        self.indices.push(3 + last_vertex_index);
        self.indices.push(0 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);
        self.indices.push(3 + last_vertex_index);

        // bottom right
        self.vertices.push(Vertex {
            pos: pos0,
            tex: tex0,
        });
        // top rigth
        self.vertices.push(Vertex {
            pos: pos1,
            tex: tex1,
        });
        // bottom left
        self.vertices.push(Vertex {
            pos: pos2,
            tex: tex2,
        });
        // top left
        self.vertices.push(Vertex {
            pos: pos3,
            tex: tex3,
        });

        self.current_batch().elements += 2;
    }

    pub fn rect(&mut self, rect: &RectF) {
        self.push_quad(
            (rect.x + rect.w, rect.y),
            (rect.x + rect.w, rect.y + rect.h),
            (rect.x, rect.y),
            (rect.x, rect.y + rect.h),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
        );
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

        self.vertices.push(Vertex {
            pos: pos0,
            tex: (0.0, 0.0),
        });
        self.vertices.push(Vertex {
            pos: pos1,
            tex: (0.0, 0.0),
        });
        self.vertices.push(Vertex {
            pos: pos2,
            tex: (0.0, 0.0),
        });
        self.current_batch().elements += 1;
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn peek_material(&mut self) -> &mut Material {
        &mut self.current_batch().material
    }

    pub fn push_material(&mut self, material: &Material) {
        let current_material = self.current_batch().material.clone();
        self.material_stack.push(current_material);
        let current: &mut DrawBatch = self.current_batch();
        if current.elements > 0 && *material != current.material {
            self.push_batch();
        }
        self.current_batch().material = material.clone();
    }

    pub fn pop_material(&mut self) {
        let material = self.material_stack.pop().unwrap();
        // let was = current.material.clone();
        let current = self.current_batch();
        if current.elements > 0 && material != current.material {
            self.push_batch();
        }
        self.current_batch().material = material.clone();
        // return was?
    }

    pub fn tex(&mut self, rect: &RectF, texture: &Texture) {
        let current = self.current_batch();
        if current.texture == *texture || current.elements == 0 {
            // reuse existing batch
            current.texture = texture.clone();
        } else {
            // create a new batch
            self.push_batch();
            self.current_batch().texture = texture.clone();
        }
        self.push_quad(
            (rect.x + rect.w, rect.y),
            (rect.x + rect.w, rect.y + rect.h),
            (rect.x, rect.y),
            (rect.x, rect.y + rect.h),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
        );
    }

    pub fn new(
        mesh: &'a mut Mesh,
        material: &'a Material,
        vertices: &'a mut Vec<Vertex>,
        indices: &'a mut Vec<u32>,
        ui: &'a mut Ui,
    ) -> Batch<'a> {
        return Batch {
            mesh,
            vertices,
            indices,
            material_stack: Vec::new(),
            batches: Vec::new(),
            default_material: material,
            ui,
        };
    }

    fn current_batch(&mut self) -> &mut DrawBatch {
        if self.batches.is_empty() {
            let value = DrawBatch {
                offset: 0,
                elements: 0,
                material: self.default_material.clone(),
                texture: Texture::default(),
                texture_sampler: TextureSampler::default(),
            };
            self.material_stack.push(self.default_material.clone());
            self.batches.push(value);
        }
        return self.batches.last_mut().unwrap();
    }
}

pub struct DrawBatch {
    offset: i64,
    elements: i64,
    material: Material,
    texture: Texture,
    texture_sampler: TextureSampler,
}
