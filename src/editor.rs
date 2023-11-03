use crate::render::V2;

const INITIAL_BUFFER_SIZE: usize = 10 * 1024;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    pub mode: Mode,
    buffer: String,
    lines: Vec<Line>,
    cursor: Pos,
}

pub struct LineIter<'a> {
    editor: &'a Editor,
    idx: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = WordIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.editor.lines.len() {
            let idx = self.idx;
            self.idx += 1;
            Some(WordIter {
                editor: self.editor,
                line: &self.editor.lines[idx],
                idx: 0,
            })
        } else {
            None
        }
    }
}

pub struct WordIter<'a> {
    editor: &'a Editor,
    line: &'a Line,
    idx: usize,
}

impl<'a> Iterator for WordIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.line.tokens.len() {
            let token = &self.line.tokens[self.idx];
            let start = token.idx();
            let end = start + token.len();
            self.idx += 1;
            Some(&self.editor.buffer[start..end])
        } else {
            None
        }
    }
}

struct Line {
    tokens: Vec<Token>,
}

impl Line {
    fn start(&self) -> usize {
        if self.tokens.is_empty() {
            0
        } else {
            self.tokens[0].idx()
        }
    }

    fn next_word(&self, idx: usize) -> Option<usize> {
        self.tokens
            .iter()
            .skip_while(|token| match token {
                Token::Word {
                    idx: token_start,
                    len: _,
                } => *token_start <= idx,
                _ => true,
            })
            .next()
            .map(|token| token.idx())
    }

    fn prev_word(&self, idx: usize) -> Option<usize> {
        self.tokens
            .iter()
            .rev()
            .skip_while(|token| match token {
                Token::Word {
                    idx: token_start,
                    len,
                } => token_start + len > idx,
                _ => true,
            })
            .next()
            .map(|token| token.idx())
    }
}

enum Token {
    Word { idx: usize, len: usize },
    Space { idx: usize, len: usize },
    Newline,
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Token::Word { idx: _, len } => *len,
            Token::Space { idx: _, len } => *len,
            Token::Newline => 1,
        }
    }

    fn idx(&self) -> usize {
        match self {
            Token::Word { idx, len: _ } => *idx,
            Token::Space { idx, len: _ } => *idx,
            Token::Newline => 0,
        }
    }
}

struct Pos {
    idx: usize,
    line: usize,
    col: usize,
}

impl Pos {
    fn next(&mut self, offset: usize) {
        self.idx += offset;
        self.col += offset;
    }

    fn prev(&mut self, offset: usize) {
        if self.col >= offset {
            self.col -= offset;
            self.idx -= offset;
        }
    }

    fn new_line(&mut self) {
        self.idx += 1;
        self.col = 0;
        self.line += 1;
    }
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            mode: Mode::Normal,
            buffer: String::with_capacity(INITIAL_BUFFER_SIZE),
            lines: vec![Line { tokens: Vec::new() }],
            cursor: Pos {
                idx: 0,
                line: 0,
                col: 0,
            },
        }
    }

    pub fn cursor(&self) -> V2 {
        V2 {
            x: self.cursor.col as f32,
            y: self.cursor.line as f32,
        }
    }

    pub fn iter(&self) -> LineIter {
        LineIter {
            editor: self,
            idx: 0,
        }
    }

    pub fn exit_insert(&mut self) {
        if self.mode == Mode::Insert {
            self.mode = Mode::Normal
        }
    }

    pub fn enter_insert(&mut self) {
        if self.mode == Mode::Normal {
            self.mode = Mode::Insert
        }
    }

    pub fn move_left(&mut self) {
        self.cursor.prev(1);
    }

    pub fn move_right(&mut self) {
        let line = &self.lines[self.cursor.line];
        if self.cursor.col < self.line_len(line) {
            self.cursor.next(1);
        }
    }

    pub fn next_word(&mut self) {
        let line = &self.lines[self.cursor.line];
        let line_next_word = line.next_word(self.cursor.idx).map(|idx| Pos {
            idx,
            line: self.cursor.line,
            col: idx - line.start(),
        });

        if let Some(next) = line_next_word.or_else(|| {
            self.next_line().and_then(|line| {
                line.tokens.iter().next().map(|token| Pos {
                    idx: token.idx(),
                    line: self.cursor.line + 1,
                    col: token.idx() - line.start(),
                })
            })
        }) {
            self.cursor = next;
        }
    }

    pub fn prev_word(&mut self) {
        let line = &self.lines[self.cursor.line];
        let line_prev_word = line.prev_word(self.cursor.idx).map(|idx| Pos {
            idx,
            line: self.cursor.line,
            col: idx - line.start(),
        });

        if let Some(next) = line_prev_word.or_else(|| {
            self.prev_line().and_then(|line| {
                line.tokens.iter().rev().next().map(|token| Pos {
                    idx: token.idx(),
                    line: self.cursor.line - 1,
                    col: token.idx() - line.start(),
                })
            })
        }) {
            self.cursor = next;
        }
    }

    fn next_line(&self) -> Option<&Line> {
        if self.lines.len() > self.cursor.line + 1 {
            Some(&self.lines[self.cursor.line + 1])
        } else {
            None
        }
    }

    fn prev_line(&self) -> Option<&Line> {
        if self.cursor.line > 0 {
            Some(&self.lines[self.cursor.line - 1])
        } else {
            None
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.line + 1 < self.lines.len() {
            self.cursor.line += 1;

            let line = &self.lines[self.cursor.line];
            let column = self.cursor.col.min(self.line_len(line));
            let line_start_idx = line.start();

            self.cursor.col = self.cursor.col.min(self.line_len(line));
            self.cursor.idx = line_start_idx + column;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;

            let line = &self.lines[self.cursor.line];
            let column = self.cursor.col.min(self.line_len(line));
            let line_start_idx = line.start();

            self.cursor.col = column;
            self.cursor.idx = line_start_idx + column;
        }
    }

    pub fn new_line(&mut self) {
        self.buffer.insert(self.cursor.idx, '\n');
        self.cursor.new_line();
        self.tokenize();
    }

    pub fn insert(&mut self, input: &str) {
        self.buffer.insert_str(self.cursor.idx, input);
        self.cursor.next(input.len());
        self.tokenize();
    }

    pub fn delete(&mut self) {
        if self.cursor.idx > 0 && self.cursor.idx <= self.buffer.len() {
            self.cursor.idx -= 1;
            self.buffer.remove(self.cursor.idx);

            // update cursor position
            if self.cursor.col > 0 {
                self.cursor.col -= 1;
            } else if self.cursor.line > 0 {
                self.cursor.line -= 1;
                self.cursor.col = self.line_len(&self.lines[self.cursor.line]);
            }
            self.tokenize();
        }
    }

    fn line_len(&self, line: &Line) -> usize {
        // move to previous line's end
        WordIter {
            editor: self,
            line,
            idx: 0,
        }
        .map(|word| word.len())
        .sum()
    }

    fn tokenize(&mut self) {
        let mut lines = Vec::new();
        let mut tokens = Vec::new();
        let mut tokenizer = Tokenizer::new();

        while let Some(token) = tokenizer.next(&self.buffer) {
            match token {
                Token::Word { .. } => {
                    tokens.push(token);
                }
                Token::Space { .. } => {
                    tokens.push(token);
                }
                Token::Newline => {
                    let new_line = tokens;
                    lines.push(Line { tokens: new_line });
                    tokens = Vec::new();
                }
            }
        }

        lines.push(Line { tokens });
        self.lines = lines;
    }
}

struct Tokenizer {
    idx: usize,
}

impl Tokenizer {
    fn new() -> Tokenizer {
        Tokenizer { idx: 0 }
    }

    fn next(&mut self, val: &str) -> Option<Token> {
        if self.idx >= val.len() {
            None
        } else {
            let current = val.as_bytes()[self.idx];
            if current == 32 {
                self.take_space(val)
            } else if current == 10 {
                self.idx += 1;
                Some(Token::Newline)
            } else {
                self.take_word(val)
            }
        }
    }

    fn take_word(&mut self, val: &str) -> Option<Token> {
        let start = self.idx;
        while self.idx < val.len() {
            let current = val.as_bytes()[self.idx];
            if current == 32 || current == 10 {
                break;
            }
            self.idx += 1;
        }
        Some(Token::Word {
            idx: start,
            len: self.idx - start,
        })
    }

    fn take_space(&mut self, val: &str) -> Option<Token> {
        let start = self.idx;
        let mut len = 0usize;
        while self.idx < val.len() {
            let current = val.as_bytes()[self.idx];
            // TODO: handle other whitespace
            if current != 32 {
                break;
            }
            self.idx += 1;
            len += 1;
        }
        Some(Token::Space { idx: start, len })
    }
}

#[cfg(test)]
mod tests {
    use super::Editor;

    #[test]
    fn tokenize_one_word() {
        let mut e = Editor::new();
        e.insert("test");

        assert_eq!(join(&e), vec!["test"])
    }

    #[test]
    fn tokenize_multiple_words() {
        let mut e = Editor::new();
        e.insert("foo bar ham");

        assert_eq!(join(&e), vec!["foo bar ham"])
    }

    #[test]
    fn tokenize_multiple_spaces() {
        let mut e = Editor::new();
        e.insert(" ham   eggs ");

        assert_eq!(join(&e), vec![" ham   eggs "])
    }

    #[test]
    fn tokenize_multiple_lines() {
        let mut e = Editor::new();
        e.insert("foo\nbar");

        assert_eq!(join(&e), vec!["foo", "bar"])
    }

    #[test]
    fn tokenize_empty_lines() {
        let mut e = Editor::new();
        e.insert("\n\n\n");

        assert_eq!(join(&e), vec!["", "", "", ""])
    }

    #[test]
    fn tokenize_empty() {
        let mut e = Editor::new();
        e.insert("");

        assert_eq!(join(&e), vec![""])
    }

    #[test]
    fn tokenize_delete_single_line() {
        let mut e = Editor::new();
        e.insert("fooo");
        e.delete();

        assert_eq!(join(&e), vec!["foo"])
    }

    #[test]
    fn tokenize_delete_multiline() {
        let mut e = Editor::new();
        e.insert("f");
        e.new_line();
        e.delete();
        e.insert("oobar");

        assert_eq!(join(&e), vec!["foobar"])
    }

    fn join(e: &Editor) -> Vec<String> {
        e.iter()
            .map(|line| line.map(|s| s.to_string()).collect::<Vec<_>>().join(""))
            .collect()
    }
}