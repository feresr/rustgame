use std::{fmt::Error, fmt::Formatter, fs::File, io::Read};

extern crate gl;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Texture {
    pub id: u32,
    width: i32,
    height: i32,
}

impl Texture {
    pub fn default() -> Self {
        return Texture {
            id: 99,
            width: 0,
            height: 0,
        };
    }
    pub fn new(path: &str) -> Self {
        print!("Creating texture path {}", path);
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            // set the texture wrapping/filtering options (on the currently bound texture object)
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
#[derive(Clone, Copy, PartialEq)]
enum TextureWrap {
    None,
    Clamp,
    Repeat,
}
#[derive(Clone, Copy, PartialEq)]
pub struct TextureSampler {
    pub filter: TextureFilter,
    wrap_x: TextureWrap,
    wrap_y: TextureWrap,
}

impl TextureSampler {
    pub fn default() -> Self {
        return TextureSampler {
            filter: TextureFilter::Linear,
            wrap_x: TextureWrap::None,
            wrap_y: TextureWrap::None,
        };
    }
}
