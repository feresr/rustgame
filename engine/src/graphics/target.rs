extern crate gl;

use std::rc::Rc;

use common::check_gl_errors;

use super::texture::Texture;
use super::texture::TextureFormat;

#[derive(Debug, Hash)]
pub struct Target {
    pub id: u32,
    pub width: i32,
    pub height: i32,
    // Target draws to these
    pub attachments: Vec<Rc<Texture>>,
}
impl Target {
    pub fn empty() -> Self {
        Target {
            id: 0,
            width: 0,
            height: 0,
            attachments: vec![],
        }
    }
    pub fn screen(w: i32, h: i32) -> Self {
        Target {
            id: 0,
            width: w,
            height: h,
            attachments: vec![],
        }
    }
    pub fn color(&self) -> Rc<Texture> {
        return self
            .attachments
            .iter()
            .find(|f| f.format != TextureFormat::DepthStencil)
            .map(|a| a.clone())
            .expect("Target has no color texture attachment");
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
            check_gl_errors!("Error trying to drop target");
        }
    }
}

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
            // gl::BindTexture(gl::TEXTURE_2D, target.id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, target.id);

            for (i, attachment) in attachments.iter().enumerate() {
                let texture = Rc::new(Texture::new(width, height, *attachment));
                let texture_id = texture.id;
                target.attachments.push(texture);
                if *attachment != TextureFormat::DepthStencil {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0 + i as u32,
                        gl::TEXTURE_2D,
                        texture_id,
                        0,
                    );
                } else {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::DEPTH_STENCIL_ATTACHMENT,
                        gl::TEXTURE_2D,
                        texture_id,
                        0,
                    );
                }
            }
        }
        return target;
    }

    pub fn clear_stencil(&self, v: i32) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::StencilMask(0xFF);
            gl::ClearStencil(v);
            gl::Clear(gl::STENCIL_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn clear(&self, color: (f32, f32, f32, f32)) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::ClearColor(color.0, color.1, color.2, color.3);
            gl::ClearDepth(1.0);
            gl::StencilMask(0xFF);
            gl::ClearStencil(0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }
}