use engine::{ecs::Component, graphics::texture::SubTexture};
use std::collections::HashMap;

pub struct Frame {
    pub image: SubTexture,
    pub duration: u32,
    pub pivot: (u32, u32),
}

pub struct Animation {
    pub frames: Vec<Frame>,
    pub name: String,
}

pub struct Sprite {
    current_frame: usize,
    frame_counter: u32,
    animations: &'static HashMap<String, Animation>,
    current_animation: &'static Animation,
    next_animation: &'static Animation,
    pub flip_x: bool,
    pub flip_y: bool,
    pub playing: bool,
}

impl Component for Sprite {}

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
        let frame = self
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
            flip_x: false,
            flip_y: false,
            playing: true,
        }
    }
}
