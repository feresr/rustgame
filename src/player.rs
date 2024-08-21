use engine::{
    ecs::{World, WorldOp},
    graphics::{
        common::RectF,
        texture::{SubTexture, Texture},
    },
};

use crate::{
    components::{
        collider::{Collider, ColliderType},
        controller::Controller,
        mover::Mover,
        position::Position,
        sprite::Sprite,
    },
    Gravity, GAME_PIXEL_HEIGHT, GAME_PIXEL_WIDTH,
};

pub struct Player;
impl Player {
    pub fn add_to_world(world: &mut impl WorldOp) {
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
                y: 0.0,
                w: 6.0,
                h: 6.0,
            },
        }));
        player.assign(Position::new(
            (GAME_PIXEL_WIDTH / 2) as i32,
            (GAME_PIXEL_HEIGHT / 2) as i32,
        ));
        player.assign(Gravity { value: 0.4f32 });
    }

    pub fn move_from(world_a: &mut World, world_b: &mut World) {
        let player_id = world_a
            .find_first::<Controller>()
            .expect("Player not present in this world, missing Controller component")
            .id;

        let mut new_player = world_b.add_entity();
        new_player.assign(
            world_a
                .extract_component::<Controller>(player_id)
                .expect("No Controller"),
        );
        new_player.assign(
            world_a
                .extract_component::<Mover>(player_id)
                .expect("No Mover"),
        );
        new_player.assign(
            world_a
                .extract_component::<Sprite>(player_id)
                .expect("No Sprite"),
        );
        new_player.assign(
            world_a
                .extract_component::<Collider>(player_id)
                .expect("No Collider"),
        );
        new_player.assign(
            world_a
                .extract_component::<Position>(player_id)
                .expect("No Position"),
        );
        new_player.assign(
            world_a
                .extract_component::<Gravity>(player_id)
                .expect("No Gravity"),
        );
    }
}
