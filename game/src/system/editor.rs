use engine::{ecs::World, graphics::batch::Batch};

use crate::{components::room::Room, content};

pub struct Editor {
    initialized: bool,
    rooms: Vec<Room>,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            initialized: false,
            rooms: Vec::new(),
        }
    }
}

impl Editor {
    pub fn update() {}

    pub fn render(&mut self, batch: &mut Batch) {
        // if !self.initialized {
        //     let ldtk = &content().ldkt;
        //     let mut i = 0;
        //     for level in ldtk.levels.iter() {
        //         let mut room = Room::from_level(level);
        //         room.prerender(batch);
        //         self.rooms.push(room);
        //         i += 1;
        //     }
        //     self.initialized = true;
        // }
        // batch.clear();
        // for room in self.rooms.iter_mut() {
        //     let room_texture = room.albedo();
        //     batch.tex(&room.rect, room_texture, (1f32, 1f32, 1f32, 1f32));
        // }
    }
}
