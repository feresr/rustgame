use engine::{
    ecs::Component,
    graphics::{
        batch::Batch,
        common::{PointF, RectF},
    },
};

use crate::Position;

#[derive(Debug, Clone)]
pub enum ColliderType {
    Rect {
        rect: RectF,
    },
    Grid {
        columns: usize,
        rows: usize,
        tile_size: usize,
        cells: Vec<bool>, // todo: should this be a []
    },
}

#[derive(Clone, PartialEq, Eq)]
pub enum Direction {
    HORIZONTAL,
    VERTICAL,
}
#[derive(Clone)]
pub struct Collision {
    pub other: u32,
    pub directions: Direction,
    pub self_velociy: glm::Vec2,
}
#[derive(Clone)]
pub struct Collider {
    pub collider_type: ColliderType,
    pub collisions: Vec<Collision>,
    debug: bool,
}
impl Collider {
    pub fn new(collider_type: ColliderType) -> Self {
        Collider {
            collider_type,
            collisions: Vec::new(),
            debug: false,
        }
    }
}
impl Component for Collider {}
impl Collider {

    pub fn check(
        &self,
        other: &Collider,
        self_position: &Position,
        other_position: &Position,
        offset: PointF,
    ) -> bool {
        return match &self.collider_type {
            ColliderType::Rect { rect: rect_a } => match &other.collider_type {
                ColliderType::Rect { rect: rect_b } => {
                    // Rect to rect collision
                    let rect_a = (rect_a
                        + PointF::new(self_position.x as f32, self_position.y as f32))
                        + offset;
                    let rect_b =
                        rect_b + PointF::new(other_position.x as f32, other_position.y as f32);
                    rect_a.intersects(&rect_b)
                }
                ColliderType::Grid {
                    columns,
                    rows,
                    tile_size,
                    cells,
                } => {
                    // Rect to grid collision
                    let distance = PointF::new(self_position.x as f32, self_position.y as f32)
                        - PointF::new(other_position.x as f32, other_position.y as f32);
                    ((rect_a + distance) + offset).intersects_grid(
                        *columns,
                        *rows,
                        *tile_size as f32,
                        cells,
                    )
                }
            },
            ColliderType::Grid {
                columns,
                rows,
                tile_size,
                cells,
            } => {
                match &other.collider_type {
                    ColliderType::Rect { rect } => {
                        // Grid to rect collision
                        println!("checking grid rect ");
                        let distance = PointF::new(self_position.x as f32, self_position.y as f32)
                            - PointF::new(other_position.x as f32, other_position.y as f32);
                        ((rect + distance) + offset).intersects_grid(
                            *columns,
                            *rows,
                            *tile_size as f32,
                            cells,
                        )
                    }
                    ColliderType::Grid {
                        columns,
                        rows,
                        tile_size,
                        cells,
                    } => {
                        // Grid to Grid collision (not supported)
                        false
                    }
                }
            }
        };
    }
}
