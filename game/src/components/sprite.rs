use engine::{
    ecs::Component,
    graphics::texture::{SubTexture, Texture},
};
use ldtk_rust::TilesetDefinition;
use std::{collections::HashMap, rc::Rc};

pub struct Frame {
    pub image: SubTexture,
    pub duration: u32,
    pub pivot: (u32, u32),
}

#[allow(dead_code)]
pub struct Tileset {
    pub texture: Rc<Texture>,
    pub normal: Rc<Texture>,
    pub tile_size: u32,
    pub rows: i64,
    pub columns: i64,
}
impl Tileset {
    pub fn from_ldtk(definition: TilesetDefinition) -> Self {
        let path = definition.rel_path.expect("Tileset is missing image path");
        // TODO
        let path = format!("game/src/assets/{}", &path);
        let texture = Rc::new(Texture::from_path(&path));
        let normal = Rc::new(Texture::from_path(&path.replace(".png", "-normal.png")));
        Self {
            texture,
            normal,
            tile_size: definition.tile_grid_size as u32,
            rows: definition.c_hei,
            columns: definition.c_wid,
        }
    }
}

pub struct Animation {
    pub frames: Vec<Frame>,
    pub name: String,
}

#[allow(dead_code)]
pub struct Sprite {
    current_frame: usize,
    frame_counter: u32,
    animations: &'static HashMap<String, Animation>,
    current_animation: &'static Animation,
    next_animation: &'static Animation,
    pub scale_x : f32,
    pub scale_y : f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub playing: bool,
}

impl Component for Sprite {}

#[allow(dead_code)]
impl Sprite {
    pub fn stop(&mut self) {
        self.playing = false;
    }
    pub fn play(&mut self, animation: &str) {
        self.playing = true;
        if self.next_animation.name == animation {
            return;
        }
        self.next_animation = self.animations.get(animation).expect("Animation not found");
    }

    pub fn pivot(&self) -> (f32, f32) {
        let frame : &Frame = self
            .current_animation
            .frames
            .get(self.current_frame)
            .expect("Missing frame");
        (frame.pivot.0 as f32, frame.pivot.1 as f32)
    }
    pub fn subtexture(&self) -> &SubTexture {
        let frame = self
            .current_animation
            .frames
            .get(self.current_frame)
            .expect("Missing frame");
        &frame.image
    }
    pub fn update_animation(&mut self, animations: &'static HashMap<String, Animation>) {
        let first_key = animations.keys().next().expect("No animations found");
        self.animations = animations;
        self.current_animation = animations.get(first_key).unwrap();
        self.next_animation = animations.get(first_key).unwrap();
    }

    pub fn tick(&mut self) {
        if !self.playing {
            return;
        }
        if self.current_animation.name != self.next_animation.name {
            self.current_animation = self.next_animation;
            self.current_frame = 0;
            self.frame_counter = 0;
            return;
        }

        let frame = self
            .current_animation
            .frames
            .get(self.current_frame)
            .expect("Missing frame");
        self.frame_counter += 1;
        if self.frame_counter >= frame.duration {
            self.frame_counter = 0;
            self.current_frame = (self.current_frame + 1) % self.current_animation.frames.len();
        }
    }
    pub fn new(animations: &'static HashMap<String, Animation>) -> Sprite {
        let first_key = animations.keys().next().expect("No animations found");
        Sprite {
            animations,
            current_animation: animations.get(first_key).unwrap(),
            next_animation: animations.get(first_key).unwrap(),
            current_frame: 0,
            frame_counter: 0,
            scale_x: 1f32,
            scale_y: 1f32,
            flip_x: false,
            flip_y: false,
            playing: true,
        }
    }
}
