use std::any::Any;
pub mod slice;
pub mod palette;
pub mod color_profile;
pub mod layer;
pub mod cel;
pub mod user;
pub mod tag;

use layer::Layer;
use cel::Cel;
use palette::Palette;
use slice::Slice;
use user::User;
use color_profile::ColorProfile;
use tag::Tags;
use std::fmt::Debug;

pub trait Chunk: Debug + Any {}
impl dyn Chunk {
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }
    
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }
}

impl Chunk for ColorProfile { }
impl Chunk for Palette {}
impl Chunk for Layer {}
impl Chunk for Cel {}
impl Chunk for Slice {}
impl Chunk for User {}
impl Chunk for Tags {}