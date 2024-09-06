use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefMut,
    path::PrefixComponent,
};

use engine::{
    ecs::{World, WorldOp},
    graphics::{batch::Batch, common::RectF},
};
use glm::vec2;

use crate::components::{
    collider::{Collider, ColliderType},
    gravity::Gravity,
    mover::Mover,
    position::{self, Position},
    rope::{Link, PointMass, Rope},
};


// in main
        // let mut rope = Rope::new();
        // let point = PointMass::new(
        //     0.0,
        //     900.0,
        //     glm::vec2(26.0, GAME_PIXEL_HEIGHT as f32 + 30.0 ),
        //     &mut self.world,
        //     0,
        // );
        // let mut prev = point;
        // for i in 1..14 {
        //     let mass = if i == 1 { 900.0 } else { 1.0 };
        //     let point = PointMass::new(
        //         0.1, 
        //         mass,
        //         glm::vec2(
        //             26.0 + i as f32 * 5.0,
        //             GAME_PIXEL_HEIGHT as f32 + 30.0 + i as f32 * 10.0,
        //         ),
        //         &mut self.world,
        //         prev,
        //     );
        //     prev = point;
        //     rope.add_point(point);
        // }

        // let mut rope_entity = self.world.add_entity();
        // rope_entity.assign(rope);

pub struct RopeSystem;

impl RopeSystem {
    pub fn update(&self, world: &mut World) {
        for rope in world.find_all::<Rope>() {
            // https://owlree.blog/posts/simulating-a-rope.html
            // https://medium.com/@szewczyk.franciszek02/rope-simulator-in-c-a595a3ef956c
            // https://matthias-research.github.io/pages/publications/posBasedDyn.pdf
            let rope = rope.component.borrow_mut();

            for iteration in 0..1 {
                for entity in world.find_all::<Link>() {
                    let mut position = world.find_component::<Position>(entity.entity_id).unwrap();
                    let mut mover = world.find_component::<Mover>(entity.entity_id).unwrap();
                    let link = entity.component.borrow();

                    let to = world.find_component::<Position>(link.to).unwrap();

                    let distance = 8.0;
                    let direction = glm::normalize(&(position.as_vec2() - to.as_vec2()));
                    position.x = to.x + (direction.x * distance) as i32;
                    position.y = to.y + (direction.y * distance) as i32;
                    let direciton_tangent = vec2(-direction.y, direction.x);

                    let velocityTangential =
                        mover.speed - glm::dot(&mover.speed, &direction) * direction;

                    let projection = (glm::dot(&mover.speed, &direciton_tangent)
                        / glm::dot(&direciton_tangent, &direciton_tangent))
                        * direciton_tangent;

                    mover.speed.x = velocityTangential.x;
                    mover.speed.y = velocityTangential.y; 
                    // mover.reminder.x = 0.0;
                    // mover.reminder.y = 0.0;

                    // let projection = (glm::dot(&mover.reminder, &direciton_tangent)
                    //     / glm::dot(&direciton_tangent, &direciton_tangent))
                    //     * direciton_tangent;
                    // mover.reminder.x = projection.x;
                    // mover.reminder.y = projection.y;
                }
            }

            // let points = rope
            //     .points
            //     .iter()
            //     .map(|point| world.find_component::<PointMass>(*point).unwrap());

            // // Apply forces and integrate positions
            // let gravity = glm::vec2(0.0, 0.2);
            // for (index, mut point_mass) in points.enumerate() {
            //     if index == 0 {
            //         continue;
            //     }
            //     let acc = (gravity) / point_mass.mass;
            //     // let mut position = world.find_component::<Position>(point_mass.id).unwrap();
            //     let new_position = 2.0 * point_mass.position - point_mass.old_position + acc;
            //     point_mass.old_position = point_mass.position;
            //     point_mass.position = new_position;
            //     // position.x = new_position.x as i32;
            //     // position.y = new_position.y as i32;
            // }

            // Calculate forces from springs
            // let rest_length = 8.0;
            // let mut points: Vec<RefMut<'_, PointMass>> = rope
            //     .points
            //     .iter()
            //     .map(|point| world.find_component::<PointMass>(*point).unwrap())
            //     .collect();
            // for iteration in 0..24 {
            //     for i in 1..points.len() {
            //         let point_a = points.get_mut(i - 1).unwrap().position;
            //         let point_b = points.get_mut(i).unwrap().position;
            //         let distance = glm::distance(&point_b, &point_a);
            //         let distance_error = distance - rest_length;

            //         let difference = point_b - point_a;
            //         let direction = glm::normalize(&difference);

            //         if i - 1 == 0 {
            //             let point_b = &mut points.get_mut(i).unwrap().position;
            //             *point_b -= direction * distance_error;
            //             // point_b_position.add(direction * distance_error);
            //         } else {
            //             let point_a = &mut points.get_mut(i - 1).unwrap().position;
            //             *point_a += 0.5 * direction * distance_error;
            //             let point_b = &mut points.get_mut(i).unwrap().position;
            //             *point_b -= 0.5 * direction * distance_error;
            //             // point_a_position.add(0.5 * direction * distance_error);
            //             // point_b_position.add(0.5 * direction * distance_error);
            //         }
            //     }
            // }

            // let mut mover = world.find_component::<Mover>(point.id).unwrap();
            // mover.speed = (point.position - point.old_position) * 1.0;
            // let mut position = world.find_component::<Position>(point.id).unwrap();
            // position.x = point.position.x as i32;
            // position.y = point.position.y as i32;
        }
    }
}
