use engine::{
    ecs::{Component, RenderWorld, UpdateWorld},
    graphics::{batch::Batch, common::RectF, material::Material, texture::TextureSampler},
};

pub struct Background {
    pub material: Material,
    pub offset: f32,
    pub radius: f32,
    pub time: f32,
    pub rect: RectF,
    pub translation_matrix: glm::Mat4,
}
impl Component for Background {
    fn update(&mut self, _: &mut UpdateWorld<'_>, entity: u32) {
        self.material.set_valuef("offset", self.offset);
        self.material.set_valuef("radius", self.radius);
        self.material.set_valuef("time", self.time);
        self.time += 0.003;
        self.offset += 0.01;
    }

    fn render(&mut self, _: &mut RenderWorld<'_>, batch: &mut Batch, entity: u32) {
        // Render the background quad
        batch.set_sampler(&TextureSampler::default());
        // Push this slightligh backwards in the z-axis so the balls render in front
        batch.push_matrix(self.translation_matrix);
        batch.push_material(&self.material);
        batch.rect(&self.rect, (1.0, 1.0, 1.0));
        batch.pop_material();
        batch.pop_matrix();
    }
}
