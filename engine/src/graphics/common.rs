pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// rust can mess with the struct layout for optimization, repr(C) avoid this 
// https://github.com/rust-lang/rust/pull/102750
#[repr(C)] 
#[derive(Debug)]
pub struct Vertex {
    pub tex: (f32, f32),
    pub pos: (f32, f32, f32),
    pub col: (f32, f32, f32),
    // col: Color,
}

pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
