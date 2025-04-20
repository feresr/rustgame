use common::Keyboard;
use engine::{
    ecs::{World, WorldOp},
    graphics::common::{PointF, RectF},
};

use crate::{
    components::{
        approach,
        button::Button,
        collider::{Collider, ColliderType},
        gravity::Gravity,
        light::Light,
        mover::Mover,
        player::{Player, COYOTE_BUFFER_TIME, JUMP_BUFFER_TIME, JUMP_SPEED, WALK_SPEED},
        position::Position,
        sprite::Sprite,
    },
    content::{self, Content},
};

pub struct PlayerSystem;
impl PlayerSystem {
    pub fn init(&self, world: &mut World) {
        let mut player = world.add_entity();
        player.assign(Player::default());
        player.assign(Mover::default());
        player.assign(Sprite::new(&Content::sprite("player")));
        player.assign(Light::with_offset(0f32, -8f32));
        player.assign(Collider::new(
            ColliderType::Rect {
                rect: RectF {
                    x: -3.0,
                    y: -8.0,
                    w: 6.0,
                    h: 8.0,
                },
            },
            true,
        ));
        player.assign(Position::new( 24 , 24 ));
        player.assign(Gravity { value: 0.3f32 });
    }

    pub fn update(&self, world: &mut World) {
        let player_entity = world.first::<Player>().expect("Player not found");

        let id = player_entity.id;
        let mut mover = player_entity.get::<Mover>();
        let mut sprite = player_entity.get::<Sprite>();
        let position = player_entity.get::<Position>();
        let collider = player_entity.get::<Collider>();
        let mut player = player_entity.get::<Player>();


        sprite.scale_x = approach(sprite.scale_x, 1.0f32, 0.10);
        sprite.scale_y = approach(sprite.scale_y, 1.0f32, 0.10);

        // TODO: coyote time
        // TODO: jump buffer time
        player.in_air = true;
        // Check if player is in air by checking if there is a collider one unit below
        player.in_air = !collider.check_all(
            id,
            &position,
            PointF { x: 0.0, y: 1f32 },
            &player_entity.world,
        );

        if player.in_air == true && player.was_in_air == false {
            // Player walked off a ledge
            player.coyote_buffer = COYOTE_BUFFER_TIME;
        }

        if !player.in_air && player.was_in_air {
            // Player just landed
            sprite.scale_x = 1.4f32;
            sprite.scale_y = 0.6f32;
        }
        player.was_in_air = player.in_air;

        if Button::is_pressed(world, "b1") {
            engine::audio().play_sound(&Content::get().tracks["jump"]);
        }

        if Keyboard::pressed(engine::Keycode::Up) || player.jump_buffer > 0 {
            if !player.in_air || player.coyote_buffer > 0 {
                // engine::audio().play_sound(&content().tracks["jump"]);
                sprite.play("jump");
                sprite.scale_x = 0.6f32;
                sprite.scale_y = 2.0f32;
                mover.speed.y = -JUMP_SPEED;
                player.jump_buffer = 0;
                player.coyote_buffer = 0;
                player.was_in_air = true;
                println!("set coyote buffer: {}", player.coyote_buffer);
            } else {
                if player.jump_buffer == 0 {
                    player.jump_buffer = JUMP_BUFFER_TIME;
                }
            }
        }

        if player.in_air {
            sprite.play("jump");
        } else {
            sprite.play("idle");
        }

        player.update();
        if !player.is_attacking() {
            if Keyboard::held(&engine::Keycode::Left) {
                mover.speed.x -= WALK_SPEED;
                sprite.flip_x = true;
                if !player.in_air {
                    sprite.play("run");
                }
            }
            if Keyboard::held(&engine::Keycode::Right) {
                mover.speed.x += WALK_SPEED;
                sprite.flip_x = false;
                if !player.in_air {
                    sprite.play("run");
                }
            }
        }
        if Keyboard::held(&engine::Keycode::Space) {
            player.attack();
        }
        if player.is_attacking() {
            sprite.play("attack");
        }

        // friction
        let x_friction = if player.in_air { 0.1 } else { 0.2 };
        mover.speed.x = approach::<f32>(mover.speed.x, 0f32, x_friction);
        // mover.speed.y = approach::<f32>(mover.speed.y, 0f32, 0.2);

        mover.speed.x = mover.speed.x.clamp(-2.0f32, 2.0f32);
    }
}
