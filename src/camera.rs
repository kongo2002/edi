use crate::render::V2;

const DELTA_TIME: f32 = 1.0 / 60.0;

pub struct Camera {
    pub pos: V2,
    pub velocity: V2,
    pub scale: f32,
    pub scale_velocity: f32,
}

impl Camera {
    pub fn update(&mut self, target: V2, target_scale: f32) {
        self.velocity = (target - self.pos) * 2.0;
        self.scale_velocity = (target_scale - self.scale) * 2.0;

        self.pos = self.pos + (self.velocity * DELTA_TIME);
        self.scale = self.scale + (self.scale_velocity * DELTA_TIME);
    }
}
