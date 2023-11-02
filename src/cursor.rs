use crate::cooldown::{Cooldown, CooldownState};
use crate::font::FONT_PIXEL_HEIGHT;
use crate::render::{Renderer, DELTA_TIME, V2, V4};

pub const CURSOR_OFFSET: f32 = 0.13;

const CURSOR_DURATION: f32 = 600.0;
const CURSOR_COOLDOWN: f32 = 400.0;

const CURSOR_ACCEL: f32 = 0.1;
const CURSOR_MIN_SPEED: f32 = 1.5;
const CURSOR_MAX_SPEED: f32 = 5.0;

const DIST_THRESHOLD_SQRD: f32 =
    CURSOR_MAX_SPEED * CURSOR_MAX_SPEED * DELTA_TIME * DELTA_TIME * 1.05;

pub struct Cursor {
    pub pos: V2,
    color: V4,
    target: V2,
    vel: V2,
    speed: f32,
    cd: Cooldown,
}

impl Cursor {
    pub fn new(color: V4) -> Cursor {
        Cursor {
            pos: (0, 0).into(),
            target: (0, 0).into(),
            vel: (0, 0).into(),
            speed: 0.0,
            color,
            cd: Cooldown::new(CURSOR_DURATION, CURSOR_COOLDOWN),
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        let cursor_size = ((FONT_PIXEL_HEIGHT as f32) / 3.0, FONT_PIXEL_HEIGHT as f32);

        renderer.render_solid_rect(self.pos, cursor_size.into(), self.color);
    }

    pub fn move_to<Pos: Into<V2>>(&mut self, pos: Pos) {
        self.target = pos.into();
    }

    pub fn update(&mut self, delta: f32) {
        let direction = self.target - self.pos;
        let dist_squared = direction.x * direction.x + direction.y * direction.y;

        if dist_squared > DIST_THRESHOLD_SQRD {
            let dir = direction / dist_squared.sqrt();

            self.speed = (self.speed + CURSOR_ACCEL)
                .max(CURSOR_MIN_SPEED)
                .min(CURSOR_MAX_SPEED);

            let velocity = dir * self.speed;
            let movement = velocity * delta;

            self.vel = velocity;
            self.pos = self.pos + movement;
        } else {
            self.pos = self.target;
            self.vel = (0, 0).into();
            self.speed = 0.0;
        }

        self.cd.update(delta);
    }

    pub fn active(&mut self) {
        self.cd.reset(CooldownState::Active);
    }

    pub fn visible(&self) -> bool {
        self.cd.state == CooldownState::Active
    }
}
