use crate::render::V2;

const INITIAL_BUFFER_SIZE: usize = 10 * 1024;

pub struct Editor {
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

    fn new_line(&mut self) {
        self.idx += 1;
        self.col = 0;
        self.line += 1;
    }
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
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

                // move to previous line's end
                let previous_end = WordIter {
                    editor: self,
                    line: &self.lines[self.cursor.line],
                    idx: 0,
                }
                .map(|word| word.len())
                .sum();

                self.cursor.col = previous_end;
            }
            self.tokenize();
        }
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
