use std::borrow::Borrow;

use engine::{
    ecs::{Component, WorldOp},
    graphics::common::PointF,
};

use crate::{Gravity, TILE_SIZE};

use super::{
    collider::{Collider, Collision},
    position::Position,
};

#[derive(Default, Clone)]
pub struct Mover {
    pub speed: glm::Vec2,
    pub reminder: glm::Vec2,
}
impl Mover {
    pub fn new(speed_x: f32, speed_y: f32) -> Self {
        Mover {
            speed: glm::vec2(speed_x, speed_y),
            reminder: glm::vec2(0.0, 0.0),
        }
    }

    fn apply_gravity<'a>(&mut self, entity: &engine::ecs::Entity<'a, impl WorldOp>) {
        let gravity = entity.get_component::<Gravity>();
        if let Some(g) = gravity {
            self.speed.y += g.value
        }
    }

    fn move_y(&mut self, amount: i32, entity: &engine::ecs::Entity<'_, impl WorldOp>) {
        if amount == 0 {
            return;
        }
        let sign_y = if amount > 0 { 1 } else { -1 };
        let mut amount = amount;
        let mut collider = entity.get_component::<Collider>().unwrap();
        for other_collider in entity.world.find_all::<Collider>() {
            if other_collider.entity_id == entity.id {
                continue;
            }
            let other = other_collider.component.borrow();
            let mut collision = false;
            while collider.borrow().check(
                &other.borrow(),
                PointF {
                    x: 0.0,
                    y: amount as f32,
                },
            ) {
                if !collision {
                    collision = true;
                    let _ = &collider.collisions.push(Collision {
                        other: other_collider.entity_id,
                        directions: super::collider::Direction::VERTICAL,
                        self_velociy: self.speed,
                    });
                    self.speed.y = 0.0;
                    self.reminder.y = 0.0;
                }
                amount -= sign_y;
            }
        }
        let mut position = entity.get_component::<Position>().unwrap();
        collider.update(&position);
        position.y += amount;
    }

    fn move_x(&mut self, amount: i32, entity: &engine::ecs::Entity<'_, impl WorldOp>) {
        if amount == 0 {
            return;
        }
        let sign_x = if amount > 0 { 1 } else { -1 };
        let mut amount = amount;
        let mut collider = entity.get_component::<Collider>().unwrap();
        for other_collider in entity.world.find_all::<Collider>() {
            if other_collider.entity_id == entity.id {
                continue;
            }
            let other = other_collider.component.borrow();
            let mut collision = false;
            while collider.borrow().check(
                &other.borrow(),
                PointF {
                    x: amount as f32,
                    y: 0.0,
                },
            ) {
                if !collision {
                    collision = true;
                    let _ = &collider.collisions.push(Collision {
                        other: other_collider.entity_id,
                        directions: super::collider::Direction::HORIZONTAL,
                        self_velociy: self.speed,
                    });
                    self.speed.x = 0.0;
                    self.reminder.x = 0.0;
                }
                amount -= sign_x;
            }
        }
        let mut position = entity.get_component::<Position>().unwrap();
        position.x += amount;
        collider.update(&position);
    }
}
impl Component for Mover {
    fn update<'a>(&mut self, entity: engine::ecs::Entity<'a, impl WorldOp>) {
        self.apply_gravity(&entity);

        let move_delta: PointF;
        {
            let mut total: glm::Vec2 = self.reminder + self.speed;
            let max_speed = TILE_SIZE - 2;
            total.x = total.x.clamp(-(max_speed as f32), max_speed as f32);
            total.y = total.y.clamp(-(max_speed as f32), max_speed as f32);
            move_delta = PointF {
                x: total.x as i32 as f32,
                y: total.y as i32 as f32,
            };
            self.reminder.x = total.x - move_delta.x;
            self.reminder.y = total.y - move_delta.y;
        }

        let mut position = entity
            .get_component::<Position>()
            .expect("Mover requires the entity to have a Position component");

        let collider = entity.get_component::<Collider>();
        if collider.is_none() {
            position.x = position.x + move_delta.x as i32;
            position.y = position.y + move_delta.y as i32;
            return;
        }
        let mut collider = collider.unwrap();
        collider.update(&position);
        collider.collisions.clear();
        drop(position);
        drop(collider);
        // Check collision X
        self.move_x(move_delta.x as i32, &entity);
        self.move_y(move_delta.y as i32, &entity);
    }
}
