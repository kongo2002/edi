use crate::render::V2;

pub struct Camera {
    pub pos: V2,
    pub scale: f32,

    velocity: V2,
    scale_velocity: f32,

    target: V2,
    target_scale: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            pos: V2::default(),
            scale: 0.3,
            velocity: V2::default(),
            scale_velocity: 0.0,
            target: V2::default(),
            target_scale: 0.3,
        }
    }

    pub fn target(&mut self, target: V2, max_line_length: f32, win_width: f32) {
        self.target = target;
        self.target_scale = (win_width / 1.0 / (max_line_length.max(1.0) * 0.75))
            .max(0.05)
            .min(0.3);
    }

    pub fn update(&mut self, delta: f32) {
        if self.target == self.pos {
            return;
        }

        let target = self.target - self.pos;
        let dist_sq = target.x * target.x + target.y * target.y;
        let dir = target / dist_sq.sqrt();

        if dist_sq < 1000.0 {
            self.pos = self.target;
        } else {
            self.velocity = dir * delta * 2.0;
            self.pos = self.pos + self.velocity;
        }

        let scale_dir = self.target_scale - self.scale;
        self.scale_velocity = scale_dir.abs().max(0.0).min(0.01) * scale_dir.signum();
        self.scale = self.scale + self.scale_velocity;
    }
}
