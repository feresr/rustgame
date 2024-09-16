extern crate gl;

use super::batch::Stencil;
use super::blend::BlendMode;
use super::material::*;
use super::mesh::*;
use super::target::*;

#[allow(dead_code)]
#[derive(Debug)]
pub struct DrawCall<'a> {
    pub mesh: &'a Mesh, // todo: use mesh to validate index/instance count
    pub material: &'a Material,
    pub target: &'a Target,
    pub index_start: i64,
    pub index_count: i64,
    pub blend: &'a BlendMode,
    pub stencil: &'a Option<Stencil>,
}

impl<'a> DrawCall<'a> {
    pub fn perform(&self) {
        // let index_count = self.mesh.indexcount;
        // if (self.index_start + self.index_count) > index_count {
        //     panic!(
        //         "Index start + count is greater than index count: {} + {} > {}",
        //         self.index_start, self.index_count, index_count
        //     );
        // }

        unsafe {
            self.material.set();
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.target.id);
            if self.target.id == 0 {
                // todo: hardcoded screen dimensions
                // gl::Viewport(0, 0, 1400, 800);
                // * 2 because of mac hdpi
                gl::Viewport(0, 0, self.target.width * 2, self.target.height * 2);
            } else {
                gl::Viewport(0, 0, self.target.width, self.target.height);
            }

            gl::BlendEquationSeparate(
                self.blend.color_op.to_gl_enum(),
                self.blend.alpha_op.to_gl_enum(),
            );
            gl::BlendFuncSeparate(
                self.blend.color_src.to_gl_enum(),
                self.blend.color_dst.to_gl_enum(),
                self.blend.alpha_src.to_gl_enum(),
                self.blend.alpha_dst.to_gl_enum(),
            );

            if let Some(s) = self.stencil {
                gl::Enable(gl::STENCIL_TEST);
                gl::StencilFunc(s.stencil_func, s.stencil_val as i32, 0xFF);
                gl::StencilOp(gl::KEEP, gl::KEEP, s.stencil_op);

                gl::ColorMask(s.color_mask, s.color_mask, s.color_mask, s.color_mask);
                gl::DepthMask(s.color_mask);

                gl::StencilMask(s.stencil_mask as u32);
            } else {
                gl::Disable(gl::STENCIL_TEST);
                gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                gl::DepthMask(gl::TRUE);
            }

            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                (core::mem::size_of::<i32>() * self.index_start as usize)
                    as *const std::os::raw::c_void,
            );
        }
    }

    pub fn new(
        mesh: &'a Mesh,
        material: &'a Material,
        target: &'a Target,
        blend: &'a BlendMode,
        stencil: &'a Option<Stencil>,
    ) -> DrawCall<'a> {
        return DrawCall {
            mesh,
            material,
            target,
            index_start: 0,
            index_count: 0,
            blend,
            stencil,
        };
    }
}
