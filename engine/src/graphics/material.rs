use super::{shader::*, texture::*};
extern crate gl;

#[derive(PartialEq, Debug, Clone)]
pub struct Material {
    shader: Shader,
    data: Vec<f32>,
    texture: Vec<Texture>,
}

impl Material {
    pub fn new(shader: Shader) -> Material {
        return Material {
            shader,
            data: Vec::new(),
            texture: vec![],
        };
    }

    pub fn set(&self) {
        unsafe {
            if !self.texture.is_empty() {
                let id = self.texture.first().unwrap().id;
                gl::BindTexture(gl::TEXTURE_2D, id);
            }
        }
        self.shader.set();
    }

    pub fn set_texture(&mut self, name: &str, texture: &Texture) {
        self.texture.push(texture.clone());
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
        }
    }

    pub fn set_sampler(&self, sampler: &TextureSampler) {
        unsafe {
            let filter = match sampler.filter {
                TextureFilter::None => gl::NONE,
                TextureFilter::Linear => gl::LINEAR,
                TextureFilter::Nearest => gl::NEAREST,
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);

            let wrap_x = match sampler.wrap_x {
                TextureWrap::Border => gl::CLAMP_TO_BORDER,
                TextureWrap::Clamp => gl::CLAMP_TO_EDGE,
                TextureWrap::Repeat => gl::REPEAT,
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_x as i32);
            let wrap_y = match sampler.wrap_y {
                TextureWrap::Border => gl::CLAMP_TO_BORDER,
                TextureWrap::Clamp => gl::CLAMP_TO_EDGE,
                TextureWrap::Repeat => gl::REPEAT,
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_y as i32);
        }
    }

    // todo: uploading data to opengl should happen before rendering (DrawCall) not here
    pub fn set_valuef(&mut self, name: &str, value: f32) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform1f(uniform.location, value);
            }
        }
    }
    pub fn set_value2f(&mut self, name: &str, value: (f32, f32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform2f(uniform.location, value.0, value.1);
            }
        }
    }
    pub fn set_value3f(&mut self, name: &str, value: (f32, f32, f32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform3f(uniform.location, value.0, value.1, value.2);
            }
        }
    }
    pub fn set_value4f(&mut self, name: &str, value: (f32, f32, f32, f32)) {
        if let Some(uniform) = self.find_uniform(name) {
            self.shader.set();
            unsafe {
                gl::Uniform4f(uniform.location, value.0, value.1, value.2, value.3);
            }
        }
    }

    fn find_uniform(&self, name: &str) -> Option<&Uniform> {
        self.shader.uniforms.iter().find(|it| it.name.eq(name))
    }
}
