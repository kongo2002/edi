#[derive(PartialEq)]
pub enum ExCmdResult {
    Command(ExCmdType),
    UnknownCommand(String),
    Quit(bool),
}

struct ExCmd {
    input: &'static str,
    typ: ExCmdType,
}

#[derive(Clone, PartialEq)]
pub enum ExCmdType {
    Quit,
    CancelQuit,
}

const ALL_COMMANDS: [ExCmd; 2] = [
    ExCmd {
        input: ":q",
        typ: ExCmdType::Quit,
    },
    ExCmd {
        input: ":cq",
        typ: ExCmdType::CancelQuit,
    },
];

pub struct CmdBuffer {
    buffer: String,
}

impl CmdBuffer {
    pub fn new() -> CmdBuffer {
        CmdBuffer {
            buffer: String::with_capacity(128),
        }
    }

    pub fn input(&mut self, input: &str) {
        self.buffer.push_str(input);
    }

    pub fn execute(&mut self) -> ExCmdResult {
        let cmd = ALL_COMMANDS
            .iter()
            .find(|cmd| cmd.input == self.buffer)
            .map(|cmd| cmd.typ.clone());

        let result = match cmd {
            Some(ExCmdType::Quit) => ExCmdResult::Quit(true),
            Some(ExCmdType::CancelQuit) => ExCmdResult::Quit(false),
            None => ExCmdResult::UnknownCommand(self.buffer.clone()),
        };

        self.reset();
        result
    }

    pub fn delete_char(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.remove(self.buffer.len() - 1);
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    pub fn as_str(&self) -> &str {
        &self.buffer
    }
}
