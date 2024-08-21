extern crate gl;

use super::material::*;
use super::mesh::*;
use super::target::*;

#[derive(Debug)]
pub struct DrawCall<'a> {
    pub mesh: &'a Mesh, // todo: use mesh to validate index/instance count
    pub material: &'a Material,
    pub target: &'a Target,
    pub index_start: i64,
    pub index_count: i64,
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

            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                (core::mem::size_of::<i32>() * self.index_start as usize)
                    as *const std::os::raw::c_void,
            );
        }
    }

    pub fn new(mesh: &'a Mesh, material: &'a Material, target: &'a Target) -> DrawCall<'a> {
        return DrawCall {
            mesh,
            material,
            target,
            index_start: 0,
            index_count: 0,
        };
    }
}
