pub mod background;
pub mod collider;
pub mod gravity;
pub mod light;
pub mod mover;
pub mod player;
pub mod position;
pub mod room;
pub mod rope;
pub mod sprite;
pub mod button;
use std::cmp::PartialOrd;
use std::ops::{Add, Sub};

// Define a trait with min and max methods
pub trait MinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

// Implement MinMax trait for i32
impl MinMax for i32 {
    fn min(self, other: i32) -> i32 {
        if self < other {
            self
        } else {
            other
        }
    }

    fn max(self, other: i32) -> i32 {
        if self > other {
            self
        } else {
            other
        }
    }
}

// Implement MinMax trait for u32
impl MinMax for u32 {
    fn min(self, other: u32) -> u32 {
        if self < other {
            self
        } else {
            other
        }
    }

    fn max(self, other: u32) -> u32 {
        if self > other {
            self
        } else {
            other
        }
    }
}

// Implement MinMax trait for u32
impl MinMax for f32 {
    fn min(self, other: f32) -> f32 {
        if self < other {
            self
        } else {
            other
        }
    }

    fn max(self, other: f32) -> f32 {
        if self > other {
            self
        } else {
            other
        }
    }
}

// Define the templated function with trait bounds
pub fn approach<T>(current: T, value: T, step: T) -> T
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + MinMax + Copy,
{
    if current < value {
        (current + step).min(value)
    } else {
        (current - step).max(value)
    }
}

pub fn create_transform(origin: &glm::Vec3, scale: &glm::Vec3) -> glm::Mat4 {
    let translate_to_origin = glm::translation(&-origin);
    let scaling_matrix = glm::scaling(scale);
    let translate_back = glm::translation(origin);
    translate_back * scaling_matrix * translate_to_origin
}
