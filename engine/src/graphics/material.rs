use std::rc::Rc;

use common::check_gl_errors;
use gl::types::GLsizei;

use super::{shader::*, texture::*};
extern crate gl;

// TODO: why can we clone a material?
#[derive(PartialEq, Debug, Clone)]
pub struct Material {
    shader: Rc<Shader>,
    data: Vec<f32>,
    textures: Vec<Rc<Texture>>, // Textures are shared by Materials an Targets, there is no clear owner, hard to know when they should be dropped, so we use Rc.
    samplers: Vec<TextureSampler>,
}

impl Material {
    pub fn with_sampler(shader: Shader, sampler: TextureSampler) -> Material {
        let texture_uniforms = shader
            .uniforms
            .iter()
            .filter(|it| it.uniform_type == UniformType::Texture2D);
        let texture_count = texture_uniforms.count();
        let textures: Vec<Rc<Texture>> = (0..texture_count).map(|_| Rc::new(Texture::default())).collect();

        return Material {
            shader : Rc::new(shader),
            data: Vec::new(),
            textures,
            samplers: vec![sampler; texture_count],
        };
    }
    pub fn new(shader: Shader) -> Material {
        let texture_uniforms = shader
            .uniforms
            .iter()
            .filter(|it| it.uniform_type == UniformType::Texture2D);
        let texture_count = texture_uniforms.count();
        let textures: Vec<Rc<Texture>> = (0..texture_count).map(|_| Rc::new(Texture::default())).collect();
        return Material {
            shader: Rc::new(shader),
            data: Vec::new(),
            textures,
            samplers: vec![TextureSampler::default(); texture_count],
        };
    }

    pub fn set(&self) {
        let mut texture_slot = 0;
        unsafe {
            self.shader.set();
            check_gl_errors!("Material::set::shader_set");
            let texture_uniforms: Vec<&Uniform> = self
                .shader
                .uniforms
                .iter()
                .filter(|it| it.uniform_type == UniformType::Texture2D)
                .collect();

            for (texture, sampler) in self.textures.iter().zip(self.samplers.iter()) {
                // select slot n
                gl::ActiveTexture(gl::TEXTURE0 + texture_slot);
                // put a texture in that slot
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                check_gl_errors!("Material::set::bind_texture");
                texture.update_sampler(sampler);

                // map uniform location to slot
                let location = texture_uniforms
                    .get(texture_slot as usize)
                    .unwrap()
                    .location;
                gl::Uniform1i(location, (texture_slot) as i32);
                texture_slot += 1;
            }
        }
    }

    pub fn has_uniform(&self, name: &str) -> bool {
        self.shader
            .uniforms
            .iter()
            .filter(|it| it.name == name)
            .count()
            > 0
    }

    pub fn set_texture(&mut self, name: &str, texture: Rc<Texture>) {
        let texture_uniforms = self
            .shader
            .uniforms
            .iter()
            .filter(|it| it.uniform_type == UniformType::Texture2D);

        for (index, uniform) in texture_uniforms.enumerate() {
            if uniform.name == *name {
                self.textures[index] = texture;
                break;
            }
        }
    }

    // TODO:
    // pub fn get_texture(&self, name : &str) -> Option<&Texture> {
    //     // todo
    //     // if let Some(uniform) = self.find_uniform(name) {
    //     // }
    // }

    // name, sampler, index
    pub fn set_sampler(&mut self, name: &str, sampler: &TextureSampler) {
        let texture_uniforms = self
            .shader
            .uniforms
            .iter()
            .filter(|it| it.uniform_type == UniformType::Texture2D);

        let mut set = false;
        for (index, uniform) in texture_uniforms.enumerate() {
            if uniform.name == *name {
                set = true;
                self.samplers[index] = sampler.clone();
                break;
            }
        }
        // TODO: only do this check in debug
        if !set {
            panic!("texture sampler not set!")
        }
    }

    // todo: uploading data to opengl should happen before rendering (DrawCall) not here
    pub fn set_valuef(&self, name: &str, value: f32) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform1f(uniform.location, value);
            }
        }
    }

    pub fn set_vector2f(&self, name: &str, value: &[f32]) {
        let uniform = self
            .find_uniform(name)
            .expect(format!("Uniform {} not found", name).as_str());
        self.shader.set();
        unsafe {
            gl::Uniform2fv(
                uniform.location,
                (value.len() / 2) as GLsizei,
                value.as_ptr(),
            );
        }
    }

    // todo, the material save uniform data as internal state and upload to GPU only on Dracall#perform()
    pub fn set_value2f(&self, name: &str, value: (f32, f32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform2f(uniform.location, value.0, value.1);
            }
        }
    }

    pub fn set_value2i(&self, name: &str, value: (i32, i32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform2i(uniform.location, value.0, value.1);
            }
        }
    }

    // todo, the material save uniform data as internal state and upload to GPU only on Dracall#perform()
    pub fn set_value3f(&mut self, name: &str, value: (f32, f32, f32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform3f(uniform.location, value.0, value.1, value.2);
            }
        }
    }
    // todo, the material save uniform data as internal state and upload to GPU only on Dracall#perform()
    pub fn set_value4f(&mut self, name: &str, value: (f32, f32, f32, f32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform4f(uniform.location, value.0, value.1, value.2, value.3);
            }
        }
    }

    // todo, the material save uniform data as internal state and upload to GPU only on Dracall#perform()
    pub fn set_matrix3x2(&mut self, name: &str, value: glm::Mat3x2) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::UniformMatrix3x2fv(uniform.location, 1, gl::FALSE, value.as_slice().as_ptr());
            }
        }
    }

    // todo, the material save uniform data as internal state and upload to GPU only on Dracall#perform()
    pub fn set_matrix4x4(&mut self, name: &str, value: &glm::Mat4x4) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::UniformMatrix4fv(uniform.location, 1, gl::FALSE, value.as_slice().as_ptr());
            }
        }
    }

    fn find_uniform(&self, name: &str) -> Option<&Uniform> {
        self.shader.uniforms.iter().find(|it| it.name.eq(name))
    }
}
