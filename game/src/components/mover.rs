use engine::ecs::Component;

#[derive(Default, Clone)]
pub struct Mover {
    pub speed: glm::Vec2,
    pub reminder: glm::Vec2,
}

impl Mover {}

impl Component for Mover {
    const CAPACITY: usize = 64;
}
