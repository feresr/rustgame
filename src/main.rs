extern crate engine;

use engine::graphics::{batch::*, common::*};

struct MyGame;

impl engine::Game for MyGame {
    fn init(&self) {
        println!("init")
    }
    fn update(&self, delta: f64) {
        println!("update")
    }
    fn render(&mut self, batch: &mut Batch) {
        let rect = RectF {
            x: -0.9,
            y: -0.9,
            w: 1.0,
            h: 1.0,
        };
        batch.rect(rect, &Color { r: 0, g: 0, b: 0 });
        let rect2 = RectF {
            x: 0.2,
            y: 0.2,
            w: 1.0,
            h: 1.0,
        };
        batch.rect(rect2, &Color { r: 0, g: 0, b: 0 });
    }
}

fn main() {
    engine::start(MyGame);
}
