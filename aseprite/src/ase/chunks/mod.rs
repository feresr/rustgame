use std::any::Any;
pub mod cel;
pub mod color_profile;
pub mod layer;
pub mod palette;
pub mod slice;
pub mod tag;
pub mod tileset;
pub mod user;

use cel::Cel;
use color_profile::ColorProfile;
use layer::Layer;
use palette::Palette;
use slice::Slice;
use std::fmt::Debug;
use tag::Tags;
use user::User;

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

impl Chunk for ColorProfile {}
impl Chunk for Palette {}
impl Chunk for Layer {}
impl Chunk for Cel {}
impl Chunk for Slice {}
impl Chunk for User {}
impl Chunk for Tags {}

