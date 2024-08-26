use engine::{
    ecs::{World, WorldOp},
    graphics::{
        common::RectF,
        texture::{SubTexture, Texture},
    },
};

use crate::components::{
    approach,
    collider::{Collider, ColliderType, Direction},
    controller::Controller,
    gravity::Gravity,
    mover::Mover,
    position::Position,
    sprite::Sprite,
};

pub struct PlayerSystem;
impl PlayerSystem {
    pub fn init(&self, world: &mut World) {
        let texture = Texture::from_path("src/blob.png");
        let mut player = world.add_entity();
        player.assign(Controller::new(8, 8));
        player.assign(Mover::default());
        player.assign(Sprite::from_sub_texture(SubTexture::new(
            &texture,
            RectF {
                x: 8f32,
                y: 0f32,
                w: 8f32,
                h: 8f32,
            },
        )));
        player.assign(Collider::new(ColliderType::Rect {
            rect: RectF {
                x: 1.0,
                y: 1.0,
                w: 6.0,
                h: 7.0,
            },
        }));
        player.assign(Position::new(72 as i32, 16 as i32));
        player.assign(Gravity { value: 0.4f32 });
    }

    pub fn update(&self, world: &mut World) {
        let player = world.find_first::<Controller>().expect("Player not found");

        let mut mover = player.get_component::<Mover>().unwrap();
        let collider = player
            .get_component::<Collider>()
            .expect("No Collider on Player");
        let mut player = player.get_component::<Controller>().unwrap();
        let keyboard = engine::keyboard();

        if keyboard.keycodes.contains(&engine::Keycode::Left) {
            mover.speed.x -= 0.8f32;
        }
        if keyboard.keycodes.contains(&engine::Keycode::Right) {
            mover.speed.x += 0.8f32;
        }
        player.in_air = true;
        for collision in &collider.collisions {
            if collision.directions == Direction::VERTICAL {
                if collision.self_velociy.y > 0f32 {
                    player.in_air = false;
                }
            }
        }

        if keyboard.keycodes.contains(&engine::Keycode::Up) && !player.in_air {
            mover.speed.y = -10f32;
        }

        // friction
        let x_friction = if player.in_air { 0.2 } else { 0.40 };
        mover.speed.x = approach::<f32>(mover.speed.x, 0.0, x_friction);
        mover.speed.y = approach::<f32>(mover.speed.y, 0.0, 0.2);

        mover.speed.x = mover.speed.x.clamp(-2.5f32, 2.5f32);
    }
}
