extern crate engine;

use engine::graphics::{batch::*, common::*};

struct MyGame;

impl engine::Game for MyGame {
    fn init(&self) {
        println!("init")
    }
    fn update(&self, delta: f64) {}

    fn render(&mut self, batch: &mut Batch) {
        let rect = RectF {
            x: -0.9,
            y: -0.9,
            w: 0.6,
            h: 0.6,
        };
        batch.rect(&rect);

        batch.circle((0.0, 0.0), 0.5, 64);

        batch.tri((0.4, 0.2), (0.9, 0.2), (0.4, 0.9))
    }
}

fn main() {
    engine::start(MyGame);
}
