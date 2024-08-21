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

#[derive(Clone)]
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
    pub entity_position: PointF,
    debug: bool,
}
impl Collider {
    pub fn new(collider_type: ColliderType) -> Self {
        Collider {
            collider_type,
            entity_position: PointF::zero(),
            collisions: Vec::new(),
            debug: false,
        }
    }
}
impl Component for Collider {
    fn render<'a>(
        &mut self,
        entity: engine::ecs::Entity<'a, impl engine::ecs::WorldOp>,
        batch: &mut Batch,
    ) {
        if !self.debug {
            return;
        }
        if let Some(pos) = entity.get_component::<Position>() {
            self.entity_position = PointF {
                x: pos.x as f32,
                y: pos.y as f32,
            }
        }
        if let ColliderType::Rect { rect } = &self.collider_type {
            batch.rect(&(rect + self.entity_position), (1.0, 0.0, 0.0));
        }
    }
}
impl Collider {
    /**
     * Checks this point against all other colliders in the world
     */
    // fn check_point(&self, x: usize, y: usize) -> bool {
    //     return match &self.collider_type {
    //         ColliderType::Rect => false,
    //         ColliderType::Grid {
    //             columns,
    //             rows,
    //             tile_size,
    //             cells,
    //         } => {
    //             // return cells[x + y * columns];
    //         }
    //     };
    // }
    pub fn update(&mut self, pos: &Position) {
        match &mut self.collider_type {
            ColliderType::Rect { rect } => {
                self.entity_position.x = pos.x as f32;
                self.entity_position.y = pos.y as f32;
            }
            ColliderType::Grid {
                columns,
                rows,
                tile_size,
                cells,
            } => {
                // todo?
            }
        }
    }

    pub fn check(&self, other: &Collider, offset: PointF) -> bool {
        return match &self.collider_type {
            ColliderType::Rect { rect: rect_a } => match &other.collider_type {
                ColliderType::Rect { rect: rect_b } => {
                    // Rect to rect collision
                    let rect_a = (rect_a + self.entity_position) + offset;
                    let rect_b = rect_b + other.entity_position;
                    rect_a.intersects(&rect_b)
                }
                ColliderType::Grid {
                    columns,
                    rows,
                    tile_size,
                    cells,
                } => {
                    // Rect to grid collision
                    ((rect_a + self.entity_position) + offset).intersects_grid(
                        *columns,
                        *rows,
                        *tile_size as f32,
                        cells,
                        offset,
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
                        ((rect + self.entity_position) + offset).intersects_grid(
                            *columns,
                            *rows,
                            *tile_size as f32,
                            cells,
                            offset,
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
