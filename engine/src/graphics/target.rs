extern crate gl;

use super::texture::Texture;
use super::texture::TextureFormat;

#[derive(Debug, Hash)]
pub struct Target {
    pub id: u32,
    pub width: i32,
    pub height: i32,
    pub attachments: Vec<Texture>, // todo maybe this should have a getter instead of being pub
}

pub static SCREEN: Target = Target {
    id: 0,
    width: 0,
    height: 0,
    attachments: vec![],
};

impl Target {
    pub fn new(width: i32, height: i32, attachments: &[TextureFormat]) -> Self {
        let mut target = Target {
            id: 0,
            width,
            height,
            attachments: Vec::new(),
        };

        unsafe {
            gl::GenFramebuffers(1, &mut target.id);
            gl::BindTexture(gl::TEXTURE_2D, target.id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, target.id);

            for (i, attachment) in attachments.iter().enumerate() {
                let texture = Texture::new(width, height, *attachment);
                target.attachments.push(texture);
                if *attachment != TextureFormat::DepthStencil {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0 + i as u32,
                        gl::TEXTURE_2D,
                        texture.id,
                        0,
                    );
                } else {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::DEPTH_STENCIL_ATTACHMENT,
                        gl::TEXTURE_2D,
                        texture.id,
                        0,
                    );
                }
            }
        }
        return target;
    }

    pub fn clear(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::ClearDepth(0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }
}

// This is copied, destroying one would destroy the underlaying fbo (reference count?)
// impl Drop for Target { fn drop(&mut self) {
//         unsafe {
//             glDeleteFramebuffers(1, &self.id);
//         }
//     }
// }
