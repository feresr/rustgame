extern crate engine;

use engine::graphics::{batch::*, common::*};

struct MyGame {
    position: (f32, f32),
    velocity : f32
}

impl engine::Game for MyGame {
    fn init(&self) {}

    fn update(&mut self) {
        self.position.0 = f32::sin(self.velocity * 1.0) * 0.9;
        self.position.1 = f32::cos(self.velocity * 5.0) * 0.25;
        self.velocity += 0.008;
    }

    fn render(&self, batch: &mut Batch) {
        let rect = RectF {
            x: -0.9,
            y: -0.9,
            w: 0.6,
            h: 0.6,
        };
        batch.rect(&rect);

        batch.circle(self.position, 0.2, 3 + (self.position.0.abs() * 10.0) as u32);

        batch.tri((0.4, 0.2), (0.9, 0.2), (0.4, 0.9));

        batch.render();
    }
}

fn main() {
    engine::start(MyGame {
        position: (0.0, 0.0),
        velocity: 0.0
    });
}
