use std::{fmt::Error, fmt::Formatter, fs::File, io::Read};

use bevy_ecs::system::Resource;

extern crate gl;

#[derive(Resource, Clone, Copy, PartialEq, Debug, Hash)]
pub struct Texture {
    pub id: u32,
    width: i32,
    height: i32,
}

impl Texture {
    pub fn default() -> Self {
        return Texture {
            id: 99, // todo: better default value
            width: 0,
            height: 0,
        };
    }

    pub fn new(width: i32, height: i32, texture_format: TextureFormat) -> Self {
        let mut texture = Texture {
            id: 0,
            width,
            height,
        };

        let mut gl_internal_format = 0;
        let mut gl_format = 0;
        let mut gl_type = 0;
        match texture_format {
            TextureFormat::R => {
                gl_internal_format = gl::RED;
                gl_format = gl::RED;
                gl_type = gl::UNSIGNED_BYTE;
            }
            TextureFormat::RG => {
                gl_internal_format = gl::RG;
                gl_format = gl::RG;
                gl_type = gl::UNSIGNED_BYTE;
            }
            TextureFormat::RGBA => {
                gl_internal_format = gl::RGBA;
                gl_format = gl::RGBA;
                gl_type = gl::UNSIGNED_BYTE;
            }
            TextureFormat::DepthStencil => {
                gl_internal_format = gl::DEPTH24_STENCIL8;
                gl_format = gl::DEPTH_STENCIL;
                gl_type = gl::DEPTH_STENCIL;
            }
            TextureFormat::None => {
                panic!("Invalid texture format {:?}", texture_format)
            }
        };
        unsafe {
            gl::GenTextures(1, &mut texture.id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl_internal_format.try_into().unwrap(),
                width,
                height,
                0,
                gl_format,
                gl_type,
                std::ptr::null(),
            );
        }
        return texture;
    }

    pub fn from_path(path: &str) -> Self {
        print!("Creating texture path {}", path);
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            // Set the texture wrapping/filtering options (on the currently bound texture object)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        // Load file into memory
        let mut f = File::open(path).expect("file not found");
        let mut contents = vec![];
        f.read_to_end(&mut contents).unwrap();

        // Load the image
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut comp: i32 = 0;
        let img: *mut u8;
        unsafe {
            img = stb_image_rust::stbi_load_from_memory(
                contents.as_mut_ptr(),
                contents.len() as i32,
                &mut x,
                &mut y,
                &mut comp,
                stb_image_rust::STBI_rgb_alpha,
            );
        }

        // Do something with it
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                x,
                y,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img as *const std::os::raw::c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        // Free the allocated memory
        unsafe {
            stb_image_rust::c_runtime::free(img);
        }
        let tex = Texture {
            id,
            width: x,
            height: y,
        };
        return tex;
    }

    pub fn loadImage(&self, path: &str) {}

    pub fn update_sampler(&self, sampler: &TextureSampler) {
        let filter = match sampler.filter {
            TextureFilter::None => gl::NONE,
            TextureFilter::Linear => gl::LINEAR,
            TextureFilter::Nearest => gl::NEAREST,
        };
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureFilter {
    None,
    Linear,
    Nearest,
}
impl std::fmt::Display for TextureFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureWrap {
    Border,
    Clamp,
    Repeat,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureSampler {
    pub filter: TextureFilter,
    pub wrap_x: TextureWrap,
    pub wrap_y: TextureWrap,
}

impl TextureSampler {
    pub fn default() -> Self {
        return TextureSampler {
            filter: TextureFilter::Linear,
            wrap_x: TextureWrap::Border,
            wrap_y: TextureWrap::Border,
        };
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextureFormat {
    None,
    R,
    RG,
    RGBA,
    DepthStencil,
}
