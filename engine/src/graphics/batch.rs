extern crate gl;

use std::f32::consts::TAU;

use bevy_ecs::prelude::*;
use imgui::TreeNodeFlags;
use imgui::Ui;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::common::*;
use super::drawcall;
use super::material::*;
use super::mesh::*;
use super::target::Target;
use super::texture::*;

// Sprite batcher used to draw text and textures
#[derive(Resource)]
pub struct Batch {
    mesh: Mesh,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    batches: Vec<DrawBatch>,
    matrix_stack: Vec<glm::Mat4>,
    material_stack: Vec<Material>,
    default_material: Material,
    // pub ui: &'a Ui,
}

impl Batch {
    pub fn render(&mut self, target: &Target, projection: &glm::Mat4) {
        if self.batches.is_empty() {
            // nothing to draw
            return;
        }

        // upload data to gpu
        self.mesh.set_data(&self.vertices);
        self.mesh.set_index_data(&self.indices);

        for batch in self.batches.iter_mut() {
            if batch.material.has_uniform("u_texture") {
                batch.material.set_texture("u_texture", &batch.texture);
                batch.material.set_sampler("u_texture", &batch.sampler);
            }
            if batch.material.has_uniform("u_matrix") {
                batch.material.set_matrix4x4("u_matrix", projection);
            }

            let mut pass = drawcall::DrawCall::new(&self.mesh, &batch.material, target);
            pass.index_start = batch.offset * 3;
            pass.index_count = batch.elements * 3;
            if pass.index_count == 0 {
                continue;
            }
            pass.perform();
        }

        // TODO: Re-implement
        // self.ui
        //     .window("Render calls")
        //     .size([400.0, 600.0], imgui::Condition::FirstUseEver)
        //     .build(|| {
        //         let mut s = DefaultHasher::new();
        //         target.hash(&mut s);
        //         let h = s.finish();
        //         let header = self
        //             .ui
        //             .collapsing_header(h.to_string(), TreeNodeFlags::DEFAULT_OPEN);
        //         if header {
        //             for (index, batch) in self.batches.iter().enumerate() {
        //                 if batch.elements == 0 {
        //                     continue;
        //                 }
        //                 let header = self
        //                     .ui
        //                     .collapsing_header(index.to_string(), TreeNodeFlags::FRAMED);
        //                 if header {
        //                     self.ui.text(format!("elements: {}", batch.elements));
        //                     self.ui.text(format!("offset: {}", batch.offset));
        //                     self.ui.text(format!("texture: {}", batch.texture.id));
        //                     self.ui.text(format!("sampler: {}", batch.sampler.filter));
        //                     self.ui.text(format!("material: {:?}", batch.material));
        //                 }
        //             }

        //             self.ui.text(format!("vertices: {:?}", self.vertices));
        //             self.ui.separator();
        //             self.ui.text(format!("indices: {:?}", self.indices));
        //         }
        //     });
    }

    pub fn set_sampler(&mut self, sampler: &TextureSampler) {
        let current = self.current_batch();
        if current.elements > 0 && *sampler != current.sampler {
            self.push_batch();
        }
        let current = self.current_batch();
        current.sampler = sampler.clone();
    }

    // Sets the current texture used for drawing.
    // Note that certain functions will override this (ex the `str` and `tex` methods)
    pub fn set_texture(&mut self, texture: &Texture) {
        let current = self.current_batch();
        if current.elements > 0 && *texture != current.texture {
            self.push_batch();
        }
        let current = self.current_batch();
        current.texture = texture.clone();
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

        let identity: glm::Mat4 = glm::Mat4::identity();
        let matrix: glm::Mat4 = *self.matrix_stack.last().unwrap_or(&identity);
        let z = -0.0;

        // bottom right
        let w = matrix * glm::vec4(pos0.0, pos0.1, z, 1.0);
        self.vertices.push(Vertex {
            pos: (w[0], w[1], w[2]),
            col : (0.0, 0.0, 0.0),
            tex: tex0,
        });
        // top rigth
        let w = matrix * glm::vec4(pos1.0, pos1.1, z, 1.0);
        self.vertices.push(Vertex {
            pos: (w[0], w[1], w[2]),
            col : (0.0, 0.0, 0.0),
            tex: tex1,
        });
        // bottom left
        let w = matrix * glm::vec4(pos2.0, pos2.1, z, 1.0);
        self.vertices.push(Vertex {
            pos: (w[0], w[1], w[2]),
            col : (0.0, 0.0, 0.0),
            tex: tex2,
        });
        // top left
        let w = matrix * glm::vec4(pos3.0, pos3.1, z, 1.0);
        self.vertices.push(Vertex {
            pos: (w[0], w[1], w[2]),
            col : (0.0, 0.0, 0.0),
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
        let mut last = (center.0 + radius, center.1, 0.0);
        let center = (center.0, center.1, 0.0);
        for i in 0..=steps {
            let radians = (i as f32 / steps as f32) * TAU;
            let next = (
                center.0 + f32::cos(radians) * radius,
                center.1 + f32::sin(radians) * radius,
                0.0,
            );
            self.tri(last, next, center);
            last = next;
        }
    }

    pub fn tri(&mut self, pos0: (f32, f32, f32), pos1: (f32, f32, f32), pos2: (f32, f32, f32)) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.push(0 + last_vertex_index);
        self.indices.push(1 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);

        self.vertices.push(Vertex {
            pos: pos0,
            col : (0.0, 0.0, 0.0),
            tex: (0.0, 0.0),
        });
        self.vertices.push(Vertex {
            pos: pos1,
            col : (0.0, 0.0, 0.0),
            tex: (0.0, 0.0),
        });
        self.vertices.push(Vertex {
            pos: pos2,
            col : (0.0, 0.0, 0.0),
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
        // todo: return was?
    }

    pub fn push_matrix(&mut self, matrix: glm::Mat4) {
        self.matrix_stack.push(matrix);
    }

    pub fn pop_matrix(&mut self) {
        self.matrix_stack.pop();
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
        mesh: Mesh,
        material: Material,
        // ui: &'a mut Ui,
    ) -> Batch {
        return Batch {
            mesh,
            vertices: Vec::new(),
            indices: Vec::new(),
            material_stack: Vec::new(),
            matrix_stack: Vec::new(),
            batches: Vec::new(),
            default_material: material,
            // ui,
        };
    }

    fn current_batch(&mut self) -> &mut DrawBatch {
        if self.batches.is_empty() {
            let value = DrawBatch {
                offset: 0,
                elements: 0,
                material: self.default_material.clone(),
                texture: Texture::default(),
                sampler: TextureSampler::default(),
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
    sampler: TextureSampler,
}
