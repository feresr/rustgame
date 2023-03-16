extern crate gl;
use super::material::*;
use super::mesh::*;

pub struct DrawCall<'a> {
    pub mesh: &'a Mesh,
    pub material: &'a Material<'a>,
    pub index_start: i64,
    pub index_count: i64,
}

impl<'a> DrawCall<'a> {
    pub fn perform(&self) {
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                0 as *const std::os::raw::c_void,
            );
        }
    }

    pub fn new(mesh: &'a Mesh, material: &'a Material<'a>) -> DrawCall<'a> {
        return DrawCall {
            mesh,
            material,
            index_start: 0,
            index_count: 0,
        };
    }
}
