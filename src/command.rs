use crate::cooldown::{Cooldown, CooldownState};

#[derive(Clone)]
pub enum CommandType {
    EnterInsert,
    EnterInsertAfter,
    EnterCommand,
    MoveLeft,
    MoveDown,
    MoveRight,
    MoveUp,
    MoveEndOfLine,
    MoveStartOfLine,
    NextWord,
    NextWordEnd,
    PrevWord,
    StartNextLine,
    StartPrevLine,
    AppendLine,
    PrependLine,
    DeleteLine,
    DeleteChar,
}

struct Command {
    input: &'static str,
    typ: CommandType,
}

const ALL_COMMANDS: [Command; 18] = [
    Command {
        input: "i",
        typ: CommandType::EnterInsert,
    },
    Command {
        input: "a",
        typ: CommandType::EnterInsertAfter,
    },
    Command {
        input: ":",
        typ: CommandType::EnterCommand,
    },
    Command {
        input: "h",
        typ: CommandType::MoveLeft,
    },
    Command {
        input: "j",
        typ: CommandType::MoveDown,
    },
    Command {
        input: "l",
        typ: CommandType::MoveRight,
    },
    Command {
        input: "k",
        typ: CommandType::MoveUp,
    },
    Command {
        input: "$",
        typ: CommandType::MoveEndOfLine,
    },
    Command {
        input: "0",
        typ: CommandType::MoveStartOfLine,
    },
    Command {
        input: "w",
        typ: CommandType::NextWord,
    },
    Command {
        input: "e",
        typ: CommandType::NextWordEnd,
    },
    Command {
        input: "b",
        typ: CommandType::PrevWord,
    },
    Command {
        input: "o",
        typ: CommandType::StartNextLine,
    },
    Command {
        input: "O",
        typ: CommandType::StartPrevLine,
    },
    Command {
        input: "A",
        typ: CommandType::AppendLine,
    },
    Command {
        input: "I",
        typ: CommandType::PrependLine,
    },
    Command {
        input: "dd",
        typ: CommandType::DeleteLine,
    },
    Command {
        input: "x",
        typ: CommandType::DeleteChar,
    },
];

pub struct Action {
    pub repeat: usize,
    pub cmd: CommandType,
}

enum CommandMatch<'a> {
    PartialMatch(Vec<&'a Command>),
    FullMatch(&'a Command),
    NoMatch,
}

impl Command {
    fn from_input(input: &str) -> CommandMatch {
        let mut candidates = Vec::new();

        for cmd in &ALL_COMMANDS {
            if cmd.input == input {
                return CommandMatch::FullMatch(cmd);
            }
            if cmd.input.starts_with(input) {
                candidates.push(cmd)
            }
        }

        if candidates.is_empty() {
            CommandMatch::NoMatch
        } else {
            CommandMatch::PartialMatch(candidates)
        }
    }
}

pub struct InputBuffer {
    buf: String,
    cd: Cooldown,
    repeat: Option<usize>,
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            buf: String::with_capacity(5),
            cd: Cooldown::new(500.0, 100.0),
            repeat: None,
        }
    }

    fn reset(&mut self) {
        self.buf.clear();
        self.cd.reset(CooldownState::Active);
        self.repeat = None;
    }

    pub fn update(&mut self, delta: f32) {
        self.cd.update(delta);
        if self.cd.state != CooldownState::Active {
            self.buf.clear();
            self.repeat = None;
        }
    }

    pub fn check(&mut self, input: &str) -> Option<Action> {
        match input.parse::<usize>().ok() {
            Some(multiplier) if multiplier > 0 || self.repeat.is_some() => {
                let current_multiplier = self.repeat.unwrap_or(0);
                let next_multiplier = if multiplier == 0 {
                    10
                } else {
                    10usize.pow((multiplier as f32).log10().abs().floor() as u32 + 1)
                };
                self.repeat = Some(current_multiplier * next_multiplier + multiplier);
                self.cd.reset(CooldownState::Active);
                None
            }
            _ => {
                self.buf.push_str(input);

                match Command::from_input(&self.buf) {
                    CommandMatch::PartialMatch(_) => {
                        self.cd.reset(CooldownState::Active);
                        None
                    }
                    CommandMatch::FullMatch(cmd) => {
                        let action = Action {
                            repeat: self.repeat.unwrap_or(1),
                            cmd: cmd.typ.clone(),
                        };
                        self.reset();
                        Some(action)
                    }
                    CommandMatch::NoMatch => {
                        self.reset();
                        None
                    }
                }
            }
        }
    }
}
