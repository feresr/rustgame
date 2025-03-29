use engine::{
    ecs::{Component, Entity, World, WorldOp},
    graphics::{
        batch::Batch,
        common::{PointF, RectF},
    },
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
    pub solid: bool,
}
impl Collider {
    pub fn new(collider_type: ColliderType, solid: bool) -> Self {
        Collider {
            collider_type,
            collisions: Vec::new(),
            solid,
        }
    }
}
impl Component for Collider {}
impl Collider {
    pub fn render(world: &World, batch: &mut Batch) {
        for collider in world.all_with::<Collider>() {
            let position = collider.get::<Position>();
            let collider = collider.get::<Collider>();
            match &collider.collider_type {
                ColliderType::Circle { radius } => {}
                ColliderType::Rect { rect } => {
                    dbg!(position.y);
                    dbg!(rect.y);
                    batch.rect(
                        &RectF {
                            x: position.x as f32 + rect.x,
                            y: position.y as f32 + rect.y,
                            w: rect.w,
                            h: rect.h,
                        },
                        (1.0, 0.0, 0.0, 0.5),
                    );
                }
                ColliderType::Grid {
                    columns,
                    rows,
                    tile_size,
                    cells,
                } => {}
            }
        }
    }

    pub fn check_all(
        &self,
        self_id: u32,
        self_position: &Position,
        offset: PointF,
        world: &World,
    ) -> bool {
        for collider_entity in world.all_with::<Collider>() {
            if collider_entity.id == self_id {
                continue;
            }
            let other_position = collider_entity.get::<Position>();
            let other_collider = collider_entity.get::<Collider>();
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
