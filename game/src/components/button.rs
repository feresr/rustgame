use engine::{
    ecs::{Component, World, WorldOp},
    graphics::common::RectF,
};

use crate::content;

use super::{collider::Collider, position::Position, sprite::Sprite};

pub struct Button {
    pub name: String,
    pub pressed: bool,
}

impl Button {
    pub fn new(name: &'static str, x: i32, y: i32, world: &mut impl WorldOp) -> u32 {
        let mut entity = world.add_entity();
        entity.assign(Button {
            name : name.to_string(),
            pressed: false,
        });
        entity.assign(Position { x, y });
        entity.assign(Collider::new(
            super::collider::ColliderType::Rect {
                rect: RectF::with_size(20f32, 22f32),
            },
            false,
        ));
        entity.assign(Sprite::new(&content().sprites["Button"]));
        entity.id
    }

    pub fn is_pressed(world: &World, name: &str) -> bool {
        let button = world
            .all_with::<Button>()
            .find(|entity| entity.get::<Button>().name == name);
        if let Some(b) = button {
            return b.get::<Button>().pressed;
        }
        return false;
    }

    pub fn update(world: &mut World) {
        for button_entity in world.all_with::<Button>() {
            let mut button = button_entity.get::<Button>();
            let button_collider = button_entity.get::<Collider>();
            let mut button_sprite = button_entity.get::<Sprite>();
            button.pressed = !button_collider.collisions.is_empty();
            if button.pressed {
                let s = &content().sprites[&String::from("ButtonPressed")];
                button_sprite.update_animation(s);
            } else {
                let s = &content().sprites[&String::from("Button")];
                button_sprite.update_animation(s);
            }
        }
    }
}

impl Component for Button {}
