extern crate gl;

use common::check_gl_errors;
use gl::types::GLenum;
use std::f32::consts::TAU;
use std::rc::Rc;

use super::blend;
use super::blend::BlendMode;
use super::common::*;
use super::drawcall;
use super::material::*;
use super::mesh::*;
use super::shader::Shader;
use super::target::Target;
use super::texture::*;
use super::FRAGMENT_SHADER_SOURCE;
use super::VERTEX_SHADER_SOURCE;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Stencil {
    pub stencil_val: u32, // Optional stencil value for masking, this is NOT what get's written into the buffer
    pub stencil_func: GLenum, // comparison logic  gl::StencilFunc(gl::EQUAL, 1, 0xFF);  “Only draw where stencil_buffer == 1”.
    pub stencil_op: GLenum,   // how to update the scencil
    pub color_mask: u8,
    pub stencil_mask: u8,
}
impl Stencil {
    pub fn disable() -> Option<Stencil> {
        None
    }

    // writes to the color buffer (uses the stencil)
    pub fn mask_leq(val: u32) -> Option<Stencil> {
        // Only pass the test if the stencil value is EQ to val
        Some(Stencil {
            stencil_val: val,
            stencil_func: gl::LEQUAL,
            stencil_op: gl::KEEP, // don't modify the stencil
            color_mask: gl::TRUE, // write the color buffer
            stencil_mask: 0x00,
        })
    }
    // writes to the color buffer (uses the stencil)
    pub fn mask_eq(val: u32) -> Option<Stencil> {
        // Only pass the test if the stencil value is EQ to val
        Some(Stencil {
            stencil_val: val,
            stencil_func: gl::EQUAL,
            stencil_op: gl::KEEP, // don't modify the stencil
            color_mask: gl::TRUE, // write the color buffer
            stencil_mask: 0x00,
        })
    }
    // writes to the stencil (leves color alone)
    pub fn decr() -> Option<Stencil> {
        // TODO: find a way to make this warn about target without stencil texture
        // Alwasys pass the test, and replace the stencil value with val
        Some(Stencil {
            stencil_val: 0,
            stencil_func: gl::ALWAYS, // always modify the stencil
            stencil_op: gl::DECR,     // substract to the scencil
            color_mask: gl::FALSE,    // don't write a color
            stencil_mask: 0xFF,
        })
    }
    pub fn increment() -> Option<Stencil> {
        // TODO: find a way to make this warn about target without stencil texture
        // Alwasys pass the test, and replace the stencil value with val
        Some(Stencil {
            stencil_val: 0,
            stencil_func: gl::ALWAYS, // always modify the stencil
            stencil_op: gl::INCR,     // add to the scencil
            color_mask: gl::FALSE,    // don't write a color
            stencil_mask: 0xFF,
        })
    }
    // writes to the stencil (leves color alone)
    pub fn write(val: u32) -> Option<Stencil> {
        // TODO: find a way to make this warn about target without stencil texture
        // Alwasys pass the test, and replace the stencil value with val
        Some(Stencil {
            stencil_val: val,
            stencil_func: gl::ALWAYS, // always modify the stencil
            stencil_op: gl::REPLACE,  // put val in the stencil
            color_mask: gl::FALSE,    // don't write a color
            stencil_mask: 0xFF,
        })
    }
    // Writes to the stencil AND the color at the same time
    pub fn write_color(val: u32) -> Option<Stencil> {
        Some(Stencil {
            stencil_val: val,
            stencil_func: gl::ALWAYS, // always modify the stencil
            stencil_op: gl::REPLACE,  // put val in the stencil
            color_mask: gl::TRUE,     // don't write a color
            stencil_mask: 0xFF,
        })
    }
}

// Sprite batcher used to draw text and textures
pub struct Batch {
    mesh: Mesh,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    batches: Vec<DrawBatch>,
    matrix_stack: Vec<glm::Mat4>,
    material_stack: Vec<Material>,
    default_material: Material,
}

pub struct DrawBatch {
    offset: i64,
    elements: i64,
    material: Material,
    texture: Rc<Texture>,
    sampler: TextureSampler,
    blend: BlendMode,
    stencil: Option<Stencil>,
}

impl Batch {
    pub fn default() -> Self {
        Batch::new(
            Mesh::new(),
            Material::new(Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)),
        )
    }
    pub fn new(
        mesh: Mesh,
        material: Material,
        // ui: &'a mut Ui,
    ) -> Batch {
        return Batch {
            mesh,
            vertices: Vec::with_capacity(512),
            indices: Vec::with_capacity(1024),
            material_stack: Vec::new(),
            matrix_stack: vec![],
            batches: Vec::new(),
            default_material: material,
        };
    }

    pub fn set_stencil(&mut self, stencil: Option<Stencil>) {
        let current = self.current_batch();
        if current.elements > 0 && stencil != current.stencil {
            self.push_batch();
        }
        self.current_batch().stencil = stencil;
    }

    /**
     * Just like render, but it uses the Target dimensions to consturct a projection matrix, which is we want most times, which is we want most times.
     */
    pub fn render(&mut self, target: &Target) {
        self.render_with_projection(target, &target.projection);
    }
    /**
     * Projection matrix: Transforms vertices from view space to clip space,
     * Normalized Device Coordinates (NDC) — a cube from -1 to 1 on all axes. So yes, we "squish" the 3D scene into this cube.
     */
    pub fn render_with_projection(&mut self, target: &Target, projection: &glm::Mat4) {
        if self.batches.is_empty() {
            // nothing to draw
            return;
        }
        // upload data to gpu
        self.mesh.set_data(&self.vertices);
        self.mesh.set_index_data(&self.indices);

        check_gl_errors!("Batch::Render pre draw");

        for batch in self.batches.iter_mut() {
            // TODO: upload a u_time uniform?
            if batch.material.has_uniform("u_texture") {
                batch
                    .material
                    .set_texture("u_texture", batch.texture.clone());
                batch.material.set_sampler("u_texture", &batch.sampler);
            }
            if batch.material.has_uniform("u_matrix") {
                batch.material.set_matrix4x4("u_matrix", projection);
            }
            if batch.material.has_uniform("u_resolution") {
                batch
                    .material
                    .set_value2i("u_resolution", (target.width, target.height));
            }
            let mut pass = drawcall::DrawCall::new(
                &self.mesh,
                &batch.material,
                target,
                &batch.blend,
                &batch.stencil,
            );
            pass.index_count = batch.elements * 3;
            if pass.index_count == 0 {
                continue;
            }
            pass.index_start = batch.offset * 3;

            pass.perform();
            check_gl_errors!("Batch::Render::Perform");
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
    pub fn set_texture(&mut self, texture: Rc<Texture>) {
        let current = self.current_batch();
        if current.elements > 0 && texture != current.texture {
            self.push_batch();
        }
        let current = self.current_batch();
        current.texture = texture;
    }

    fn push_batch(&mut self) {
        let current = self.current_batch();
        let value = DrawBatch {
            offset: current.offset + current.elements,
            elements: 0,
            material: current.material.clone(),
            texture: current.texture.clone(),
            ..*current
        };
        self.batches.push(value);
    }

    // todo: naive implementation, avoid duplicating verices
    pub fn cube(&mut self, center: (f32, f32), size: f32, color: (f32, f32, f32, f32)) {
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
            0,
            0,
            255,
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
            0,
            0,
            255,
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
            0,
            0,
            255,
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
            0,
            0,
            255,
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
            0,
            0,
            255,
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
            0,
            0,
            255,
        );

        self.pop_matrix();
        self.pop_matrix();
    }

    pub fn circle_fan(
        &mut self,
        center: (f32, f32),
        points: &Vec<(f32, f32)>,
        color: (f32, f32, f32, f32),
    ) {
        // TODO: use fan instead of triangles
        for i in 0..points.len() {
            let next = points[(i + 1) % points.len()];
            let pos0 = (points[i].0, points[i].1, 0.0);
            let pos1 = (next.0, next.1, 0.0);
            self.tri(pos0, pos1, (center.0, center.1, 0.0), color);
        }
    }

    pub fn rect(&mut self, rect: &RectF, color: (f32, f32, f32, f32)) {
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
            0,
            0,
            255,
        );
    }

    pub fn circle(
        &mut self,
        center: (f32, f32),
        radius: f32,
        steps: u32,
        color: (f32, f32, f32, f32),
    ) {
        let mut last = (center.0 + radius, center.1, 0.0);
        let center = (center.0, center.1, 0.0);
        let radians = (1 as f32 / steps as f32) * TAU;
        for i in 0..=steps {
            let next = (
                center.0 + f32::cos(radians * i as f32) * radius,
                center.1 + f32::sin(radians * i as f32) * radius,
                0.0,
            );
            self.tri(last, next, center, color);
            last = next;
        }
    }

    pub fn sprite(&mut self, rect: &RectF, subtexture: &SubTexture, color: (f32, f32, f32, f32)) {
        let current = self.current_batch();
        if current.texture == subtexture.texture || current.elements == 0 {
            // reuse existing batch
            current.texture = subtexture.texture.clone();
        } else {
            // create a new batch
            self.push_batch();
            self.current_batch().texture = subtexture.texture.clone();
        }
        // current.texture = subtexture.texture.clone();
        let x = subtexture.source.x / subtexture.texture.width as f32;
        let y = subtexture.source.y / subtexture.texture.height as f32;
        let w = (subtexture.source.w) / subtexture.texture.width as f32;
        let h = (subtexture.source.h) / subtexture.texture.height as f32;
        
        self.push_quad(
            (rect.x + rect.w, rect.y, 0.0),
            (rect.x + rect.w, rect.y + rect.h, 0.0),
            (rect.x, rect.y, 0.0),
            (rect.x, rect.y + rect.h, 0.0),
            (x + w, y),
            (x + w, y + h),
            (x, y),
            (x, y + h),
            color,
            color,
            color,
            color,
            255,
            0,
            0,
        );
    }

    pub fn tex(&mut self, rect: &RectF, texture: Rc<Texture>, color: (f32, f32, f32, f32)) {
        let draw_batch = self.current_batch();
        if draw_batch.texture == texture || draw_batch.elements == 0 {
            // reuse existing batch
            draw_batch.texture = texture;
        } else {
            // create a new batch
            self.push_batch();
            self.current_batch().texture = texture.clone();
        }
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
            255,
            0,
            0,
        );
    }

    pub fn tri(
        &mut self,
        pos0: (f32, f32, f32),
        pos1: (f32, f32, f32),
        pos2: (f32, f32, f32),
        color: (f32, f32, f32, f32),
    ) {
        let last_vertex_index = self.vertices.len() as u32;
        self.indices.extend([
            0 + last_vertex_index,
            1 + last_vertex_index,
            2 + last_vertex_index,
        ]);
        self.vertices.reserve(3);
        self.push_vertex(pos0, (0.0, 0.0), color, 0, 0, 255);
        self.push_vertex(pos1, (0.0, 0.0), color, 0, 0, 255);
        self.push_vertex(pos2, (0.0, 0.0), color, 0, 0, 255);
        self.current_batch().elements += 1;
    }

    pub fn peek_material(&mut self) -> &mut Material {
        &mut self.current_batch().material
    }

    pub fn set_blend(&mut self, blend_mode: BlendMode) {
        if self.current_batch().blend == blend_mode {
            return;
        }
        self.push_batch(); // TODO blend stack?
        self.current_batch().blend = blend_mode;
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
        if self.matrix_stack.is_empty() {
            self.matrix_stack.push(matrix);
        } else {
            let current: &glm::Mat4 = self.matrix_stack.last().unwrap();
            self.matrix_stack.push(current * matrix);
        }
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
    }

    fn push_vertex(
        &mut self,
        pos: (f32, f32, f32),
        tex: (f32, f32),
        col: (f32, f32, f32, f32),
        mult: u8,
        wash: u8,
        fill: u8,
    ) {
        if !self.matrix_stack.is_empty() {
            let mut position = glm::vec4(pos.0, pos.1, pos.2, 1.0);
            // TODO: this is slow - move to GPU?!
            let matrix: &glm::Mat4 = self.matrix_stack.last().unwrap();
            position = matrix * position;
            self.vertices.push(Vertex {
                pos: (position.x, position.y, position.z),
                col,
                tex,
                typ: (mult, wash, fill, 0),
            });
        } else {
            self.vertices.push(Vertex {
                pos,
                col,
                tex,
                typ: (mult, wash, fill, 0),
            });
        }
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
        color0: (f32, f32, f32, f32),
        color1: (f32, f32, f32, f32),
        color2: (f32, f32, f32, f32),
        color3: (f32, f32, f32, f32),
        mult: u8,
        wash: u8,
        fill: u8,
    ) {
        let last_vertex_index = self.vertices.len() as u32;

        self.indices.extend([
            1 + last_vertex_index,
            0 + last_vertex_index,
            3 + last_vertex_index,
            0 + last_vertex_index,
            2 + last_vertex_index,
            3 + last_vertex_index,
        ]);

        // bottom right
        self.vertices.reserve(4);
        self.push_vertex(pos0, tex0, color0, mult, wash, fill);
        // top right
        self.push_vertex(pos1, tex1, color1, mult, wash, fill);
        // bottom left
        self.push_vertex(pos2, tex2, color2, mult, wash, fill);
        // top left
        self.push_vertex(pos3, tex3, color3, mult, wash, fill);

        self.current_batch().elements += 2;
    }

    fn current_batch(&mut self) -> &mut DrawBatch {
        if self.batches.is_empty() {
            let value = DrawBatch {
                offset: 0,
                elements: 0,
                material: self.default_material.clone(),
                texture: Rc::new(Texture::default()),
                sampler: TextureSampler::default(),
                blend: blend::NORMAL,
                stencil: None,
            };
            self.material_stack.push(self.default_material.clone());
            self.batches.push(value);
        }
        return self.batches.last_mut().unwrap();
    }
}
