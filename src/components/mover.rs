use std::borrow::{Borrow, BorrowMut};

use engine::{
    ecs::{Component, RenderWorld, UpdateWorld, WorldOp},
    graphics::{batch::Batch, common::PointF},
};

use crate::{Gravity, TILE_SIZE};

use super::{approach, collider::Collider, position::Position};

#[derive(Default)]
pub struct Mover {
    pub speed: glm::Vec2,
    reminder: glm::Vec2,
}
impl Mover {
    pub fn new(speed_x: f32, speed_y: f32) -> Self {
        Mover {
            speed: glm::vec2(speed_x, speed_y),
            reminder: glm::vec2(0.0, 0.0),
        }
    }
}
impl Component for Mover {
    fn update<'a>(&mut self, world: &'a mut UpdateWorld<'_>, entity: u32) {
        // todo: clamp dx dy to tilesize
        // self.dx = self.dx.clamp(-(TILE_SIZE as f32), TILE_SIZE as f32);
        // self.dy = self.dy.clamp(-(TILE_SIZE as f32), TILE_SIZE as f32);
        let gravity = world.find_component::<Gravity>(entity);
        if let Some(g) = gravity {
            self.speed.y -= g.value
        }

        let mut total: glm::Vec2 = self.reminder + self.speed;
        let max_speed = TILE_SIZE - 2;
        total.x = total.x.clamp(-(max_speed as f32), max_speed as f32);
        total.y = total.y.clamp(-(max_speed as f32), max_speed as f32);
        let offset: PointF = PointF {
            x: total.x as i32 as f32,
            y: total.y as i32 as f32,
        };
        self.reminder.x = total.x - offset.x;
        self.reminder.y = total.y - offset.y;

        let position = world.find_component::<Position>(entity);
        if let Some(mut p) = position {
            // Has collider?
            if let Some(mut my_collider) = world.find_component::<Collider>(entity) {
                my_collider.update(&p);

                // Check collision X
                let colliders = world.find_all::<Collider>();
                // If a collision is found reduce dx dy
                let mut offset_x = offset.x as i32;
                for collider in colliders {
                    if collider.entity_id == entity {
                        continue;
                    }
                    let other = collider.component.borrow();
                    while my_collider.borrow().check(
                        &other.borrow(),
                        PointF {
                            x: offset_x as f32,
                            y: 0.0,
                        },
                    ) {
                        // (my_collider.on_collision)();
                        if offset_x == 0 {
                            break;
                        };
                        offset_x = approach(offset_x, 0, 1);
                        self.speed.x = 0.0;
                        self.reminder.x = 0.0;
                    }
                }
                p.x += offset_x;
                my_collider.update(&p);

                // Check collision Y
                let mut offset_y = offset.y as i32;
                let colliders = world.find_all::<Collider>();
                for collider in colliders {
                    if collider.entity_id == entity {
                        continue;
                    }
                    let other = collider.component.borrow();
                    while my_collider.borrow().check(
                        &other.borrow(),
                        PointF {
                            x: 0.0,
                            y: offset_y as f32,
                        },
                    ) {
                        // (my_collider.on_collision)();
                        if offset_y == 0 {
                            break;
                        };
                        offset_y = approach(offset_y, 0, 1);
                        self.speed.y = 0.0;
                        self.reminder.y = 0.0;
                    }
                }
                // offset.y = offset_y as f32;
                p.y += offset_y;
                if p.y < 0 {
                    p.y = 0;
                }
                my_collider.update(&p);
            } else {
                p.x = p.x + self.speed.x as i32;
                p.y = p.y + self.speed.y as i32;
            }
        }

        // floor
    }

    fn render<'a>(&mut self, world: &'a mut RenderWorld<'_>, batch: &mut Batch, entity: u32) {}
}
