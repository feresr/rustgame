use std::{arch::aarch64::vabds_f32, ops};

use gl::SET;
use sdl2::rect::Point;

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
    pub col: (f32, f32, f32, f32),
    pub typ: (u8, u8, u8, u8), // mult wash fill (pad)
}

#[derive(Clone, Copy, Debug)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}
impl PointF {
    pub fn new(x: f32, y: f32) -> Self {
        PointF { x, y }
    }
    pub fn zero() -> Self {
        PointF { x: 0.0, y: 0.0 }
    }
}

impl From<&PointF> for glm::Vec2 {
    fn from(value: &PointF) -> Self {
        return glm::vec2(value.x, value.y);
    }
}
impl From<PointF> for (f32, f32) {
    fn from(value: PointF) -> Self {
        return (value.x, value.y);
    }
}

#[derive(Debug, Clone, Default)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl ops::Sub<PointF> for PointF {
    type Output = PointF;

    fn sub(self, rhs: PointF) -> Self::Output {
        PointF {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl ops::Add<PointF> for &PointF {
    type Output = PointF;

    fn add(self, rhs: PointF) -> Self::Output {
        PointF {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
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

pub struct EdgeF {
    pub a: PointF,
    pub b: PointF,
}
impl EdgeF {
    pub fn translate(&mut self, offset: &glm::Vec2) {
        self.a.x += offset.x;
        self.b.x += offset.x;
        self.a.y += offset.y;
        self.b.y += offset.y;
    }
    pub fn normal(&self) -> glm::Vec2 {
        let v = glm::Vec2::from(&self.a) - glm::Vec2::from(&self.b);
        glm::vec2(-v.y, v.x)
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
    
    pub fn translate_by(&mut self, point: &PointF) {
        self.x += point.x;
        self.y += point.y;
    }

    pub fn contains(&self, point: &PointF) -> bool {
        point.x >= self.x && point.x <= self.x + self.w && point.y >= self.y && point.y <= self.y + self.h
    }

    pub fn points(&self) -> [PointF; 4] {
        return [
            PointF::new(self.x, self.y),
            PointF::new(self.x + self.w, self.y),
            PointF::new(self.x, self.y + self.h),
            PointF::new(self.x + self.w, self.y + self.h),
        ];
    }
    pub fn center(&self) -> PointF {
        return PointF {
            x: self.x + self.w / 2f32,
            y: self.y + self.h / 2f32,
        };
    }
    pub fn scale(&mut self, factor: f32) {
        let w = self.w;
        let h = self.h;
        let new_w = w * factor;
        let new_h = h * factor;
        let diff_w = new_w - w;
        let diff_h = new_h - h;

        self.x -= diff_w / 2f32;
        self.y -= diff_h / 2f32;
        self.w = new_w;
        self.h = new_h;
    }

    /**
     * (x,y) ------------(a) ------ (x + w, h)
     *   |                              |
     *   |                              |
     *  (d)                            (b)
     *   |                              |
     *   |                              |
     * (x, y + h) ------ (c) ------ (x + w, y + h)
     */
    pub fn edges(&self) -> [EdgeF; 4] {
        return [
            EdgeF {
                a: PointF::new(self.x, self.y),
                b: PointF::new(self.x + self.w, self.y),
            },
            EdgeF {
                a: PointF::new(self.x + self.w, self.y),
                b: PointF::new(self.x + self.w, self.y + self.h),
            },
            EdgeF {
                a: PointF::new(self.x + self.w, self.y + self.h),
                b: PointF::new(self.x, self.y + self.h),
            },
            EdgeF {
                a: PointF::new(self.x, self.y + self.h),
                b: PointF::new(self.x, self.y),
            },
        ];
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
    ) -> bool {
        // Calculate the grid cell boundaries that the rect overlaps
        let start_x = ((self.x) / tile_size) as usize;
        let end_x = ((self.x + self.w) / tile_size).ceil() as usize;
        let start_y = ((self.y) / tile_size) as usize;
        let end_y = ((self.y + self.h) / tile_size).ceil() as usize;

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
