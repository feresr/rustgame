use super::{
    shader::*,
    texture::{self, *},
};
extern crate gl;

#[derive(PartialEq, Debug, Clone)]
pub struct Material {
    shader: Shader,
    data: Vec<f32>,
    textures: Vec<Texture>,
    samplers: Vec<TextureSampler>,
}

impl Material {
    pub fn new(shader: Shader) -> Material {
        let texture_uniforms = shader
            .uniforms
            .iter()
            .filter(|it| it.uniform_type == UniformType::Texture2D);
        let texture_count = texture_uniforms.count();
        return Material {
            shader,
            data: Vec::new(),
            textures: vec![Texture::default(); texture_count],
            samplers: vec![TextureSampler::default(); texture_count],
        };
    }

    pub fn set(&self) {
        let mut texture_slot = 0;
        self.shader.set();
        unsafe {
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

    pub fn set_texture(&mut self, name: &str, texture: &Texture) {
        let texture_uniforms = self
            .shader
            .uniforms
            .iter()
            .filter(|it| it.uniform_type == UniformType::Texture2D);

        for (index, uniform) in texture_uniforms.enumerate() {
            if uniform.name == *name {
                self.textures[index] = texture.clone();
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

        for (index, uniform) in texture_uniforms.enumerate() {
            if uniform.name == *name {
                self.samplers[index] = sampler.clone();
                break;
            }
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
