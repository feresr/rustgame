use engine::ecs::{Component, World, WorldOp};

use super::button::Button;

pub struct Light {}

impl Light {
    pub fn new() -> Self {
        Light {}
    }
}
impl Component for Light {}

pub struct LightSwitch {
    pub button_name: &'static str,
    switch_state: bool,
    old_button_state: bool,
}
impl Component for LightSwitch {}

impl LightSwitch {
    pub fn new(button_name: &'static str) -> Self {
        LightSwitch {
            button_name,
            switch_state: false,
            old_button_state: false,
        }
    }
    pub fn update(world: &mut World) {
        let mut turn_on: Vec<u32> = Vec::new();
        let mut turn_off: Vec<u32> = Vec::new();
        for entity in world.all_with::<LightSwitch>() {
            let mut ls = entity.get::<LightSwitch>();
            let is_pressed = Button::is_pressed(world, ls.button_name);
            if is_pressed && !ls.old_button_state {
                ls.switch_state = !ls.switch_state;
            }
            ls.old_button_state = is_pressed;
            if ls.switch_state {
                if let None = entity.has::<Light>() {
                    turn_on.push(entity.id);
                }
            } else {
                if let Some(f) = entity.has::<Light>() {
                    turn_off.push(entity.id);
                }
            }
        }

        turn_on
            .iter()
            .for_each(|id| world.entity_mut(*id).assign(Light {}));
        turn_off.iter().for_each(|id| {
            world.entity_mut(*id).unassign::<Light>();
        })
    }
}
