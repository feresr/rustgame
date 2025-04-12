use engine::ecs::{Component, World, WorldOp};

use super::button::Button;

pub struct Light {
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Light {
    pub fn with_offset(x: f32, y: f32) -> Self {
        return Light {
            offset_x: x,
            offset_y: y,
        };
    }
    pub fn new() -> Self {
        Light {
            offset_x: 0f32,
            offset_y: 0f32,
        }
    }
}
impl Component for Light {}

pub struct LightSwitch {
    pub button_name: &'static str,
    turned_on: bool,
    old_button_state: bool,
}
impl Component for LightSwitch {}

impl LightSwitch {
    pub fn new(button_name: &'static str) -> Self {
        LightSwitch {
            button_name,
            turned_on: false,
            old_button_state: false,
        }
    }
    pub fn update(world: &mut World) {
        let mut turn_on: Vec<u32> = Vec::new();
        let mut turn_off: Vec<u32> = Vec::new();
        for light_switch_entity in world.all_with::<LightSwitch>() {
            let mut ls = light_switch_entity.get::<LightSwitch>();
            let is_pressed = Button::is_pressed(world, ls.button_name);
            if is_pressed && !ls.old_button_state {
                ls.turned_on = !ls.turned_on;
            }
            ls.old_button_state = is_pressed;
            if ls.turned_on {
                dbg!("turned on");
                if let None = light_switch_entity.has::<Light>() {
                    dbg!("turning light");
                    turn_on.push(light_switch_entity.id);
                }
            } else {
                if light_switch_entity.has::<Light>().is_some() {
                    turn_off.push(light_switch_entity.id);
                }
            }
        }

        turn_on
            .iter()
            .for_each(|id| world.entity_mut(*id).assign(Light::new()));
        turn_off.iter().for_each(|id| {
            world.entity_mut(*id).unassign::<Light>();
        })
    }
}
