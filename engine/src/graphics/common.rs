use std::ops;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// rust can mess with the struct layout for optimization, repr(C) avoid this
// https://github.com/rust-lang/rust/pull/102750
// #[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub tex: (f32, f32),
    pub pos: (f32, f32, f32),
    pub col: (f32, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}
impl PointF {
    pub fn zero() -> Self {
        PointF { x: 0.0, y: 0.0 }
    }
}
#[derive(Debug, Clone)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl ops::Add<PointF> for &RectF {
    type Output = RectF;

    fn add(self, rhs: PointF) -> Self::Output {
        RectF {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            w: self.w,
            h: self.h,
        }
    }
}
impl ops::Add<PointF> for RectF {
    type Output = RectF;

    fn add(self, rhs: PointF) -> Self::Output {
        RectF {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            w: self.w,
            h: self.h,
        }
        // self.x = self.x + rhs.x;
        // self.y = self.y + rhs.y;
        // self.w = self.w;
        // self.h = self.h;
        // self
    }
}
impl RectF {
    pub fn with_size(w: f32, h: f32) -> Self {
        Self {
            x: 0f32,
            y: 0f32,
            w,
            h,
        }
    }

    pub fn intersects(&self, other: &RectF) -> bool {
        return self.x + self.w > other.x
            && self.y + self.h > other.y
            && self.x < other.x + other.w
            && self.y < other.y + other.h;
    }

    pub fn intersects_grid(
        &self,
        columns: usize,
        rows: usize,
        tile_size: f32,
        cells: &[bool],
        offset: PointF,
    ) -> bool {
        // Calculate the grid cell boundaries that the rect overlaps
        let start_x = ((self.x + offset.x) / tile_size) as usize;
        let end_x = ((self.x + offset.x + self.w) / tile_size).ceil() as usize;
        let start_y = ((self.y + offset.y) / tile_size) as usize;
        let end_y = ((self.y + offset.y + self.h) / tile_size).ceil() as usize;

        // Iterate over each grid cell in the overlapping region
        for y in start_y..end_y.min(rows) {
            for x in start_x..end_x.min(columns) {
                let index = y * columns + x;

                // Check if the grid cell is occupied
                if cells[index] {
                    let cell_rect = RectF {
                        x: x as f32 * tile_size,
                        y: y as f32 * tile_size,
                        w: tile_size,
                        h: tile_size,
                    };

                    // Check for intersection with the current grid cell
                    if self.intersects(&cell_rect) {
                        return true; // Collision detected
                    }
                }
            }
        }

        false // No collision detected
    }
}
