use engine::{
    ecs::{Component, Entity, WorldOp},
    graphics::{batch::Batch, common::RectF},
};
use rand::Rng;

use crate::{Gravity, GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH};

use super::{
    collider::{self, Collider, ColliderType},
    mover::Mover,
    position::Position,
};

pub struct Ball {
    pub r: u32,
    pub spawned_a_new: u32,
}
impl Ball {
    pub fn spawn_new(world: &mut impl WorldOp) {
        let mut ball = world.add_entity();
        ball.assign(Ball {
            r: 2,
            spawned_a_new: 0,
        });

        let mut rng = rand::thread_rng();
        ball.assign(Mover::new(
            2.0f32 * (-0.5 + rng.gen::<f32>()),
            2.0f32 * (-0.5 + rng.gen::<f32>()),
        ));
        ball.assign(Position::new(320 / 2, 170 / 2));
        ball.assign(Gravity { value: 0.1f32 });

        ball.assign(Collider::new(ColliderType::Rect {
            rect: RectF::with_size(2f32, 2f32),
        }));
    }
}
impl Component for Ball {
    fn update<'a>(&mut self, entity: engine::ecs::Entity<'a, impl WorldOp>) {
        let mut collided_with_screen = false;
        {
            let mut ball_mover = entity.get_component::<Mover>().unwrap();
            let mut ball_position = entity.get_component::<Position>().unwrap();
            if (ball_position.x + self.r as i32) > GAME_PIXEL_WIDTH as i32
                || (ball_position.x - self.r as i32) < 0
            {
                ball_mover.speed.x *= -1f32;
                ball_mover.reminder.x *= -1f32;
                collided_with_screen = true;
            }
            if (ball_position.y + self.r as i32) > GAME_PIXEL_HEIGHT as i32 {
                ball_position.y = GAME_PIXEL_HEIGHT as i32 - self.r as i32;
                ball_mover.speed.y = -ball_mover.speed.y;
                ball_mover.reminder.y = -ball_mover.reminder.y;
                collided_with_screen = true;
            }
            if (ball_position.y - self.r as i32) < 0 {
                ball_mover.speed.y = -ball_mover.speed.y;
                ball_position.y = self.r as i32;
                collided_with_screen = true;
            }
        }

        let collided;
        {
            let mut collider = entity.get_component::<Collider>().unwrap();
            let mut mover = entity.get_component::<Mover>().unwrap();
            collided = !collider.collisions.is_empty();
            for collision in collider.collisions.drain(..) {
                match collision.directions {
                    collider::Direction::HORIZONTAL => {
                        mover.speed.x = -collision.self_velociy.x;
                    }
                    collider::Direction::VERTICAL => {
                        mover.speed.y = -collision.self_velociy.y;
                    }
                }
            }
        }
        if collided && self.spawned_a_new > 0 {
            Ball::spawn_new(entity.world);
            self.spawned_a_new -= 1;
        }
    }

    fn render<'a>(&mut self, entity: Entity<'a, impl WorldOp>, batch: &mut Batch) {
        let position = entity.get_component::<Position>().unwrap();
        let rect = RectF {
            x: position.x as f32,
            y: position.y as f32,
            w: self.r as f32,
            h: self.r as f32,
        };
        batch.rect(&rect, (1.0, 1.0, 0.0));
    }
}
