use engine::ecs::Component;

pub struct Light {}
impl Light {
    pub fn new() -> Self {
        Light {}
    }
}
impl Component for Light {}
