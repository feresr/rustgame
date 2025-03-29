use engine::ecs::component::Component;

#[derive(Default, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        return Position { x, y };
    }
    pub fn set(&mut self, position: glm::Vec2) {
        self.x = position.x as i32;
        self.y = position.y as i32;
    }
    pub fn add(&mut self, position: glm::Vec2) {
        self.x += position.x as i32;
        self.y += position.y as i32;
    }
    pub fn as_vec2(&self) -> glm::Vec2 {
        return glm::vec2(self.x as f32, self.y as f32);
    }
    pub fn as_vec3(&self) -> glm::Vec3 {
        return glm::vec3(self.x as f32, self.y as f32, 0f32);
    }
}
impl Component for Position {
    const CAPACITY: usize = 64;
}
