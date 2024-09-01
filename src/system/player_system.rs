use engine::{
    ecs::{World, WorldOp},
    graphics::common::{PointF, RectF},
};

use crate::{
    components::{
        approach,
        collider::{Collider, ColliderType},
        controller::Player,
        gravity::Gravity,
        mover::Mover,
        position::Position,
        sprite::Sprite,
    },
    content::content,
};

pub struct PlayerSystem;
impl PlayerSystem {
    pub fn init(&self, world: &mut World) {
        let mut player = world.add_entity();
        player.assign(Player::new(8, 8));
        player.assign(Mover::default());
        player.assign(Sprite::new(&content().sprites["player"]));
        player.assign(Collider::new(ColliderType::Rect {
            rect: RectF {
                x: -3.0,
                y: -8.0,
                w: 6.0,
                h: 8.0,
            },
        }));
        player.assign(Position::new(72 as i32, 52 as i32));
        player.assign(Gravity { value: 0.2f32 });
    }

    pub fn update(&self, world: &mut World) {
        let player_entity = world.find_first::<Player>().expect("Player not found");

        let id = player_entity.id;
        let mut mover = player_entity.get_component::<Mover>().unwrap();
        let mut sprite = player_entity.get_component::<Sprite>().unwrap();
        let position = player_entity.get_component::<Position>().unwrap();
        let collider = player_entity
            .get_component::<Collider>()
            .expect("No Collider on Player");
        let mut player = player_entity.get_component::<Player>().unwrap();
        let keyboard = engine::keyboard();

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
        if keyboard.keycodes.contains(&engine::Keycode::Up) && !player.in_air {
            engine::audio().play_sound(&content().tracks["jump"]);
            sprite.play("jump");
            mover.speed.y = -10f32;
        }
        if player.in_air {
            sprite.play("jump");
        } else {
            sprite.play("idle");
        }

        player.update();
        if !player.is_attacking() {
            if keyboard.keycodes.contains(&engine::Keycode::Left) {
                mover.speed.x -= 0.6f32;
                sprite.flip_x = true;
                sprite.play("run");
            }
            if keyboard.keycodes.contains(&engine::Keycode::Right) {
                mover.speed.x += 0.6f32;
                sprite.flip_x = false;
                sprite.play("run");
            }
        }
        if keyboard.keycodes.contains(&engine::Keycode::Space) {
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
