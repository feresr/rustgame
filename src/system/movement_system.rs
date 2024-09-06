use std::{borrow::BorrowMut, cell::RefMut};

use engine::{
    ecs::{World, WorldOp},
    graphics::common::PointF,
};

use crate::{
    components::{
        collider::{Collider, Collision},
        gravity::Gravity,
        mover::Mover,
        position::Position,
    },
    TILE_SIZE,
};

pub struct MovementSystem;
impl MovementSystem {
    pub fn update(&self, world: &mut World) {
        for mover in world.find_all::<Mover>() {
            let entity_id = mover.entity_id;
            let mut mover = mover.component.borrow_mut();
            let gravity = world.find_component::<Gravity>(entity_id);
            if let Some(g) = gravity {
                if mover.speed.y < 0.0 {
                    // falling down
                    mover.speed.y += g.value * 1.0f32;
                } else {
                    mover.speed.y += g.value * 1.4f32
                }
            }

            {
                let mut total: glm::Vec2 = mover.reminder + mover.speed;
                let max_speed = TILE_SIZE - 2;
                total.x = total.x.clamp(-(max_speed as f32), max_speed as f32);
                total.y = total.y.clamp(-(max_speed as f32), max_speed as f32);

                mover.speed.x = total.x as i32 as f32;
                mover.speed.y = total.y as i32 as f32;
                mover.reminder.x = total.x - mover.speed.x;
                mover.reminder.y = total.y - mover.speed.y;
            }

            let mut position = world
                .find_component::<Position>(entity_id)
                .expect("Mover requires the entity to have a Position component");

            let collider = world.find_component::<Collider>(entity_id);
            if collider.is_none() {
                // Entity has no collider, move it and return early
                position.x = position.x + mover.speed.x as i32;
                position.y = position.y + mover.speed.y as i32;
                continue;
            }
            let mut collider = collider.unwrap();
            collider.collisions.clear();
            MovementSystem::move_x(
                mover.speed.x as i32,
                entity_id,
                &mut collider,
                &mut position,
                &mut mover,
                world,
            );
            MovementSystem::move_y(
                mover.speed.y as i32,
                entity_id,
                &mut collider,
                &mut position,
                &mut mover,
                world,
            );
        }
    }

    fn move_x(
        amount: i32,
        entity: u32,
        collider: &mut RefMut<Collider>,
        position: &mut RefMut<Position>,
        mover: &mut RefMut<Mover>,
        world: &World,
    ) {
        if amount == 0 {
            return;
        }

        let sign_x = if amount > 0 { 1 } else { -1 };
        let mut amount = amount;
        for wrapper in world.find_all::<Collider>() {
            if wrapper.entity_id == entity {
                continue;
            }
            let other_position = world.find_component::<Position>(wrapper.entity_id).unwrap();
            let other_collider = wrapper.component.borrow();
            let mut collision = false;
            while collider.check(
                &other_collider,
                &position,
                &other_position,
                PointF {
                    x: amount as f32,
                    y: 0.0,
                },
            ) {
                if !collision {
                    collision = true;
                    let _ = collider.collisions.push(Collision {
                        other: wrapper.entity_id,
                        directions: crate::components::collider::Direction::HORIZONTAL,
                        self_velocity: mover.speed,
                    });
                    mover.speed.x = 0.0;
                    mover.reminder.x = 0.0;
                }
                amount -= sign_x;
            }
        }
        position.x += amount;
    }

    pub fn move_y(
        amount: i32,
        entity: u32,
        collider: &mut RefMut<Collider>,
        position: &mut RefMut<Position>,
        mover: &mut RefMut<Mover>,
        world: &World,
    ) {
        if amount == 0 {
            return;
        }

        let sign_y = if amount > 0 { 1 } else { -1 };
        let mut amount = amount;
        for wrapper in world.find_all::<Collider>() {
            if wrapper.entity_id == entity {
                continue;
            }
            let other_position = world.find_component::<Position>(wrapper.entity_id).unwrap();
            let other_collider = wrapper.component.borrow();
            let mut collision = false;
            while collider.check(
                &other_collider,
                &position,
                &other_position,
                PointF {
                    x: 0.0,
                    y: amount as f32,
                },
            ) {
                if !collision {
                    collision = true;
                    let _ = collider.collisions.push(Collision {
                        other: wrapper.entity_id,
                        directions: crate::components::collider::Direction::VERTICAL,
                        self_velocity: mover.speed,
                    });
                    mover.speed.y = 0.0;
                    mover.reminder.y = 0.0;
                }
                amount -= sign_y;
            }
        }
        position.y += amount;
    }
}
