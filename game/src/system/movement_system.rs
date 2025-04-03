use std::cell::RefMut;

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
    }, game_state::TILE_SIZE,
};

pub struct MovementSystem;
impl MovementSystem {
    pub fn update(&self, world: &mut World) {
        // Clear all previous collisions
        world.all_with::<Collider>().for_each(|collider_entity| {
            collider_entity.get::<Collider>().collisions.clear();
        });

        // For everything that moves...
        for mover_entity in world.all_with::<Mover>() {
            let mut mover = mover_entity.get::<Mover>();
            let gravity = mover_entity.has::<Gravity>();
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

            let mut position = mover_entity.get::<Position>();

            let collider = mover_entity.has::<Collider>();
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
                mover_entity.id,
                &mut collider,
                &mut position,
                &mut mover,
                world,
            );
            MovementSystem::move_y(
                mover.speed.y as i32,
                mover_entity.id,
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
        let sign_x = amount.signum();
        let mut amount = amount;
        for collider_entity in world.all_with::<Collider>() {
            if collider_entity.id == entity {
                continue;
            }
            let other_position = collider_entity.get::<Position>();
            let mut other_collider = collider_entity.get::<Collider>();
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
                    collider.collisions.push(Collision {
                        other: collider_entity.id,
                        directions: crate::components::collider::Direction::HORIZONTAL,
                        self_velocity: mover.speed,
                    });
                    other_collider.collisions.push(Collision {
                        other: entity,
                        directions: crate::components::collider::Direction::VERTICAL,
                        self_velocity: glm::Vec2::new(0f32, 0f32),
                    });

                    if other_collider.solid {
                        collision = true;
                        mover.speed.x = 0.0;
                        mover.reminder.x = 0.0;
                    } else {
                        break;
                    }
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
        let sign_y = amount.signum();
        let mut amount = amount;
        for collider_entity in world.all_with::<Collider>() {
            if collider_entity.id == entity {
                continue;
            }
            let other_position = collider_entity.get::<Position>();
            let mut other_collider = collider_entity.get::<Collider>();
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
                    collider.collisions.push(Collision {
                        other: collider_entity.id,
                        directions: crate::components::collider::Direction::VERTICAL,
                        self_velocity: mover.speed,
                    });
                    other_collider.collisions.push(Collision {
                        other: entity,
                        directions: crate::components::collider::Direction::VERTICAL,
                        self_velocity: glm::Vec2::new(0f32, 0f32),
                    });

                    if other_collider.solid {
                        collision = true;
                        mover.speed.y = 0.0;
                        mover.reminder.y = 0.0;
                    } else {
                        break;
                    }
                }
                amount -= sign_y;
            }
        }
        position.y += amount;
    }
}
