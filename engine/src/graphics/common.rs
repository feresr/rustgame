pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug)]
pub struct Vertex {
    pub pos: (f32, f32),
    // tex: (f32, f32),
    // col: Color,
}

pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
