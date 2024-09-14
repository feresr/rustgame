#![allow(dead_code)]
use engine::{
    ecs::{Component, World, WorldOp},
    graphics::common::RectF,
};

use crate::{
    components::collider::{Collider, ColliderType},
    content,
};

use super::{gravity::Gravity, mover::Mover, position::Position, sprite::Sprite};

// Linked list?
#[allow(dead_code)]
pub struct PointMass {
    pub id: u32,
    pub mass: f32,
    pub position: glm::Vec2,
    pub old_position: glm::Vec2,
}

pub struct Link {
    pub to: u32,
}
impl Component for Link {}

impl PointMass {
    pub fn new(gravity: f32, mass: f32, position: glm::Vec2, world: &mut World, prev: u32) -> u32 {
        let mut entity = world.add_entity();
        let id = entity.id;
        entity.assign(Position::new(position.x as i32, position.y as i32));
        entity.assign(Sprite::new(&content().sprites["rope"]));
        entity.assign(Gravity { value: gravity });
        entity.assign(Mover {
            speed: glm::vec2(0.2, 0.0),
            reminder: glm::vec2(0.0, 0.0),
        });
        entity.assign(Collider::new(ColliderType::Rect {
            rect: RectF {
                x: -1f32,
                y: -1f32,
                w: 2.0,
                h: 2.0,
            },
        }));
        if prev != 0 {
            entity.assign(Link { to: prev });
        }
        let point = Self {
            id: entity.id,
            mass,
            position,
            old_position: position,
        };
        entity.assign(point);
        return id;
    }
}
impl Component for PointMass {}

#[allow(dead_code)]
pub struct Rope {
    pub points: Vec<u32>,
}
impl Rope {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
    pub fn add_point(&mut self, point: u32) {
        self.points.push(point);
    }
}
impl Component for Rope {}
