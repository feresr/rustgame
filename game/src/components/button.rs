use engine::{
    ecs::{Component, World, WorldOp},
};

use crate::content;

use super::{collider::Collider, sprite::Sprite};

pub struct Button {
    pub name: String,
    pub pressed: bool,
}

impl Button {
    pub fn is_pressed(world: &World, name: &str) -> bool {
        let button = world.all_with::<Button>().find(|entity| {
            let b = entity.get::<Button>();
            return b.name == name;
        });
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
