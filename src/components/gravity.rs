use engine::ecs::Component;

pub struct Gravity {
    pub value: f32,
}
impl Component for Gravity {
    const CAPACITY: usize = 32;
}
