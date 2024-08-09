use engine::{
    ecs::{Component, UpdateWorld, WorldOp},
    graphics::common::{PointF, RectF},
};
use glm::Vec2;

use crate::Position;

#[derive(Debug)]
pub enum ColliderType {
    Rect {
        rect: RectF,
    },
    Grid {
        columns: usize,
        rows: usize,
        tile_size: usize,
        cells: Vec<bool>,
    },
}

#[derive(Debug)]
pub struct Collider {
    pub collider_type: ColliderType,
    // pub on_collision: Box<dyn Fn(&dyn WorldOp) -> usize>,
    entity_position: PointF,
}
impl Collider {
    pub fn with_callback(collider_type: ColliderType, callback: fn()) -> Self {
        Collider {
            collider_type,
            entity_position: PointF::zero(),
            // on_collision: callback,
        }
    }
    pub fn new(collider_type: ColliderType) -> Self {
        Collider {
            collider_type,
            entity_position: PointF::zero(),
            // on_collision: || {},
        }
    }
}
impl Component for Collider {
    fn update<'a>(&mut self, world: &'a mut engine::ecs::UpdateWorld<'_>, entity: u32) {
        if let Some(pos) = world.find_component::<Position>(entity) {
            match &mut self.collider_type {
                ColliderType::Rect { rect } => {
                    // rect.x = pos.x as f32;
                    // rect.y = pos.y as f32;
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
    }

    fn render<'a>(
        &mut self,
        world: &'a mut engine::ecs::RenderWorld<'_>,
        batch: &mut engine::graphics::batch::Batch,
        entity: u32,
    ) {
        if let ColliderType::Rect { rect } = &self.collider_type {
            batch.rect(&(rect + self.entity_position), (1.0, 1.0, 0.0));
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
