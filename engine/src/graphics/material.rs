use super::shader::*;
extern crate gl;

pub struct Material<'a> {
    shader: &'a Shader,
    data: Vec<f32>,
}

impl <'a> Material<'a> {
    pub fn new(shader: &'a Shader) -> Material<'a> {
        return Material {
            shader,
            data: Vec::new(),
        };
    }

    pub fn set_valuef(&mut self, name: &str, value: f32) {
        let u = self.shader.uniforms.iter().find(|it| it.name.eq(name));
        if let Some(uniform) = u {
            println!("{}", uniform.name);
            unsafe {
                gl::Uniform1f(uniform.location, value);
            }
        }
        
    }
    pub fn set_value2f(&mut self, _name: &str, _value: (f32, f32)) {}
    pub fn set_value3f(&mut self, _name: &str, _value: (f32, f32, f32)) {}
    pub fn set_value4f(&mut self, _name: &str, _value: (f32, f32, f32, f32)) {}
}
