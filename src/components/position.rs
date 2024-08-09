use engine::{
    ecs::{Component, RenderWorld, UpdateWorld},
    graphics::batch::Batch,
};

#[derive(Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        return Position { x, y };
    }
}
impl Component for Position {
    fn update<'a>(&mut self, world: &'a mut UpdateWorld<'_>, entity: u32) {}
    fn render<'a>(&mut self, world: &'a mut RenderWorld<'_>, batch: &mut Batch, entity: u32) {}
}
