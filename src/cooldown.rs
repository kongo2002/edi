#[derive(PartialEq)]
pub enum CooldownState {
    Active,
    OnCooldown,
}

pub struct Cooldown {
    pub state: CooldownState,
    cooldown: f32,
    duration: f32,
    current: f32,
}

impl Cooldown {
    pub fn new(duration: f32, cooldown: f32) -> Cooldown {
        Cooldown {
            cooldown,
            duration,
            current: 0.0,
            state: CooldownState::Active,
        }
    }

    pub fn reset(&mut self, state: CooldownState) {
        self.state = state;
        match self.state {
            CooldownState::Active => self.current = self.duration,
            CooldownState::OnCooldown => self.current = self.cooldown,
        }
    }

    pub fn update(&mut self, delta: f32) {
        match self.state {
            CooldownState::Active => {
                if self.current > delta {
                    self.current -= delta;
                } else {
                    self.current = self.cooldown;
                    self.state = CooldownState::OnCooldown;
                }
            }
            CooldownState::OnCooldown => {
                if self.current > delta {
                    self.current -= delta;
                } else {
                    self.current = self.duration;
                    self.state = CooldownState::Active;
                }
            }
        }
    }
}
