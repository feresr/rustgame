extern crate gl;

use super::material::*;
use super::mesh::*;

#[derive(Debug)]
pub struct DrawCall<'a> {
    pub mesh: &'a Mesh,
    pub material: &'a Material,
    pub index_start: i64,
    pub index_count: i64,
}

impl<'a> DrawCall<'a> {
    pub fn perform(&self) {
        self.material.set();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                (core::mem::size_of::<i32>() * self.index_start as usize)
                    as *const std::os::raw::c_void,
            );
        }
    }

    pub fn new(mesh: &'a Mesh, material: &'a Material) -> DrawCall<'a> {
        return DrawCall {
            mesh,
            material,
            index_start: 0,
            index_count: 0,
        };
    }
}
