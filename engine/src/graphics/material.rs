use super::shader::*;
extern crate gl;

#[derive(PartialEq, Debug)]
pub struct Material {
    shader: Shader,
    data: Vec<f32>,
}

impl<'a> Material {
    pub fn new(shader: Shader) -> Material {
        return Material {
            shader,
            data: Vec::new(),
        };
    }

    pub fn set(&self) {
        self.shader.set();
    }

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
