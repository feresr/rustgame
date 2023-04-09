extern crate gl;

use bevy_ecs::prelude::*;
use imgui::TreeNodeFlags;
use imgui::Ui;
use std::f32::consts::TAU;

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
}


pub(crate) trait ImGuiable {
    fn render_imgui(&self, imgGui: &Ui);
}


impl ImGuiable for Batch {
    fn render_imgui(&self, imgui: &Ui) {
        imgui
            .window("Render calls")
            .size([400.0, 600.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let header = imgui.collapsing_header("header", TreeNodeFlags::DEFAULT_OPEN);
                if header {
                    for (index, batch) in self.batches.iter().enumerate() {
                        if batch.elements == 0 {
                            continue;
                        }
                        let header =
                            imgui.collapsing_header(index.to_string(), TreeNodeFlags::FRAMED);
                        if header {
                            imgui.text(format!("elements: {}", batch.elements));
                            imgui.text(format!("offset: {}", batch.offset));
                            imgui.text(format!("texture: {}", batch.texture.id));
                            imgui.text(format!("sampler: {}", batch.sampler.filter));
                            imgui.text(format!("material: {:?}", batch.material));
                        }
                    }

                    imgui.text(format!("vertices: {:?}", self.vertices));
                    imgui.separator();
                    imgui.text(format!("indices: {:?}", self.indices));
                }
            });
    }
}

impl Batch {
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
            matrix_stack: vec![glm::Mat4::identity()],
            batches: Vec::new(),
            default_material: material,
            // ui,
        };
    }

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
            if batch.material.has_uniform("u_resolution") {
                // println!("setting u_res to {}x{}", target.width, target.height);
                batch
                    .material
                    .set_value2i("u_resolution", (target.width, target.height));
            }

            let mut pass = drawcall::DrawCall::new(&self.mesh, &batch.material, target);
            pass.index_start = batch.offset * 3;
            pass.index_count = batch.elements * 3;
            if pass.index_count == 0 {
                continue;
            }
            pass.perform();
        }
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

    // todo: naive implementation, avoid duplicating verices
    pub fn cube(&mut self, center: (f32, f32), size: f32, color: (f32, f32, f32)) {
        let rect = RectF {
            x: center.0 - size / 2.0,
            y: center.1 - size / 2.0,
            w: size,
            h: size,
        };
        self.push_quad(
            (rect.x + rect.w, rect.y, -size / 2.0),
            (rect.x + rect.w, rect.y + rect.h, -size / 2.0),
            (rect.x, rect.y, -size / 2.0),
            (rect.x, rect.y + rect.h, -size / 2.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );

        self.push_quad(
            (rect.x + rect.w, rect.y, size / 2.0),
            (rect.x + rect.w, rect.y + rect.h, size / 2.0),
            (rect.x, rect.y, size / 2.0),
            (rect.x, rect.y + rect.h, size / 2.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );
        self.push_matrix(glm::rotate(
            &glm::identity(),
            1.5708,
            &glm::vec3(0.0, 1.0, 0.0),
        ));
        self.push_quad(
            (rect.x + rect.w, rect.y, -size / 2.0),
            (rect.x + rect.w, rect.y + rect.h, -size / 2.0),
            (rect.x, rect.y, -size / 2.0),
            (rect.x, rect.y + rect.h, -size / 2.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );

        self.push_quad(
            (rect.x + rect.w, rect.y, size / 2.0),
            (rect.x + rect.w, rect.y + rect.h, size / 2.0),
            (rect.x, rect.y, size / 2.0),
            (rect.x, rect.y + rect.h, size / 2.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );
        self.push_matrix(glm::rotate(
            &glm::identity(),
            1.5708,
            &glm::vec3(1.0, 0.0, 0.0),
        ));
        self.push_quad(
            (rect.x + rect.w, rect.y, -size / 2.0),
            (rect.x + rect.w, rect.y + rect.h, -size / 2.0),
            (rect.x, rect.y, -size / 2.0),
            (rect.x, rect.y + rect.h, -size / 2.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );

        self.push_quad(
            (rect.x + rect.w, rect.y, size / 2.0),
            (rect.x + rect.w, rect.y + rect.h, size / 2.0),
            (rect.x, rect.y, size / 2.0),
            (rect.x, rect.y + rect.h, size / 2.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );

        self.pop_matrix();
        self.pop_matrix();
    }

    pub fn rect(&mut self, rect: &RectF, color: (f32, f32, f32)) {
        self.push_quad(
            (rect.x + rect.w, rect.y, 0.0),
            (rect.x + rect.w, rect.y + rect.h, 0.0),
            (rect.x, rect.y, 0.0),
            (rect.x, rect.y + rect.h, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            color,
            color,
            color,
            color,
        );
    }

    pub fn circle(&mut self, center: (f32, f32), radius: f32, steps: u32, color: (f32, f32, f32)) {
        let mut last = (center.0 + radius, center.1, 0.0);
        let center = (center.0, center.1, 0.0);
        for i in 0..=steps {
            let radians = (i as f32 / steps as f32) * TAU;
            let next = (
                center.0 + f32::cos(radians) * radius,
                center.1 + f32::sin(radians) * radius,
                0.0,
            );
            self.tri(last, next, center, color);
            last = next;
        }
    }

    pub fn tex(&mut self, rect: &RectF, texture: &Texture, color: (f32, f32, f32)) {
        let current = self.current_batch();
        if current.texture == *texture || current.elements == 0 {
            // reuse existing batch
            current.texture = texture.clone();
        } else {
            // create a new batch
            self.push_batch();
            self.current_batch().texture = texture.clone();
        }
        // todo! not all texture should be z= -1 (Demo purposes delete)
        self.push_quad(
            (rect.x + rect.w, rect.y, 0.0),
            (rect.x + rect.w, rect.y + rect.h, 0.0),
            (rect.x, rect.y, 0.0),
            (rect.x, rect.y + rect.h, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (0.0, 1.0),
            color,
            color,
            color,
            color,
        );
    }

    pub fn tri(
        &mut self,
        pos0: (f32, f32, f32),
        pos1: (f32, f32, f32),
        pos2: (f32, f32, f32),
        color: (f32, f32, f32),
    ) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.push(0 + last_vertex_index);
        self.indices.push(1 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);

        let matrix: glm::Mat4 = *self.matrix_stack.last().unwrap();

        self.push_vertex(&matrix, pos0, (0.0, 0.0), color);
        self.push_vertex(&matrix, pos1, (0.0, 0.0), color);
        self.push_vertex(&matrix, pos2, (0.0, 0.0), color);
        self.current_batch().elements += 1;
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
        let current: glm::Mat4 = *self.matrix_stack.last().unwrap();
        self.matrix_stack.push(current * matrix);
    }

    pub fn pop_matrix(&mut self) {
        self.matrix_stack.pop();
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.vertices.clear();
        self.indices.clear();
        self.material_stack.clear();
        self.matrix_stack.clear();
        self.matrix_stack.push(glm::Mat4::identity());
    }

    fn push_vertex(
        &mut self,
        matrix: &glm::Mat4,
        position: (f32, f32, f32),
        tex: (f32, f32),
        col: (f32, f32, f32),
    ) {
        let w = matrix * glm::vec4(position.0, position.1, position.2, 1.0);
        self.vertices.push(Vertex {
            pos: (w[0], w[1], w[2]),
            col,
            tex,
        });
    }

    fn push_quad(
        &mut self,
        pos0: (f32, f32, f32),
        pos1: (f32, f32, f32),
        pos2: (f32, f32, f32),
        pos3: (f32, f32, f32),
        tex0: (f32, f32),
        tex1: (f32, f32),
        tex2: (f32, f32),
        tex3: (f32, f32),
        color0: (f32, f32, f32),
        color1: (f32, f32, f32),
        color2: (f32, f32, f32),
        color3: (f32, f32, f32),
    ) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.push(1 + last_vertex_index);
        self.indices.push(0 + last_vertex_index);
        self.indices.push(3 + last_vertex_index);
        self.indices.push(0 + last_vertex_index);
        self.indices.push(2 + last_vertex_index);
        self.indices.push(3 + last_vertex_index);

        let matrix: glm::Mat4 = *self.matrix_stack.last().unwrap();
        // bottom right
        self.push_vertex(&matrix, pos0, tex0, color0);
        // top right
        self.push_vertex(&matrix, pos1, tex1, color1);
        // bottom left
        self.push_vertex(&matrix, pos2, tex2, color2);
        // top left
        self.push_vertex(&matrix, pos3, tex3, color3);

        self.current_batch().elements += 2;
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
