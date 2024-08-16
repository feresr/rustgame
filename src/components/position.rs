use engine::ecs::Component;

#[derive(Default, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        return Position { x, y };
    }
}
impl Component for Position {}
