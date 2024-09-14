use engine::{
    ecs::{Component, World, WorldOp},
    graphics::common::{PointF, RectF},
};

use crate::Position;
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ColliderType {
    Circle {
        radius: f32,
    },
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
#[allow(dead_code)]
#[derive(Clone)]
pub struct Collision {
    pub other: u32,
    pub directions: Direction,
    pub self_velocity: glm::Vec2,
}
#[derive(Clone)]
pub struct Collider {
    pub collider_type: ColliderType,
    pub collisions: Vec<Collision>,
}
impl Collider {
    pub fn new(collider_type: ColliderType) -> Self {
        Collider {
            collider_type,
            collisions: Vec::new(),
        }
    }
}
impl Component for Collider {}
impl Collider {
    pub fn check_all(
        &self,
        self_id: u32,
        self_position: &Position,
        offset: PointF,
        world: &World,
    ) -> bool {
        for wrapper in world.find_all::<Collider>() {
            if wrapper.entity_id == self_id {
                continue;
            }
            let other_position = world.find_component::<Position>(wrapper.entity_id).unwrap();
            let other_collider = wrapper.component.borrow();
            if self.check(&other_collider, self_position, &other_position, offset) {
                return true;
            }
        }
        false
    }
    pub fn check(
        &self,
        other: &Collider,
        self_position: &Position,
        other_position: &Position,
        offset: PointF,
    ) -> bool {
        return match &self.collider_type {
            ColliderType::Circle { radius: _radius_a } => match &other.collider_type {
                ColliderType::Circle { radius: _radius_b } => {
                    return true;
                }
                ColliderType::Rect { rect: _rect_b } => {
                    return true;
                }
                ColliderType::Grid {
                    columns: _,
                    rows: _,
                    tile_size: _,
                    cells: _,
                } => {
                    // TODO: Implement grid to circle collision
                    return true;
                }
            },
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
                ColliderType::Circle { radius: _radius_b } => {
                    return true;
                }
            },
            ColliderType::Grid {
                columns: _,
                rows: _,
                tile_size: _,
                cells: _,
            } => {
                //
                return false;
            }
        };
    }
}
