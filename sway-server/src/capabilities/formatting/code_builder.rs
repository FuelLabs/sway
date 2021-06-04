use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use lspower::lsp::{Position, Range, TextEdit};

use super::code_line::CodeLine;

#[derive(Debug)]
pub struct CodeBuilder {
    tab_size: u32,
    indent_level: u32,
    edits: Vec<CodeLine>,
}

impl CodeBuilder {
    pub fn new(tab_size: u32) -> Self {
        Self {
            tab_size,
            indent_level: 0,
            edits: vec![],
        }
    }

    fn get_unfinished_code_line_or_new(&mut self) -> CodeLine {
        match self.edits.last() {
            Some(code_line) => {
                if code_line.is_completed {
                    CodeLine::default()
                } else {
                    self.edits.pop().unwrap()
                }
            }
            None => CodeLine::default(),
        }
    }

    pub fn format_and_add(&mut self, line: &str) {
        if is_comment(line) {
            return self.add_line(CodeLine::new(line.into()));
        }

        let mut code_line = self.get_unfinished_code_line_or_new();

        // handle multiline string
        if code_line.is_string {
            code_line.push_char('\n');
        }

        let line = if !code_line.is_string {
            line.trim()
        } else {
            line
        };

        let mut iter = line.chars().enumerate().peekable();

        loop {
            if let Some((_, current_char)) = iter.next() {
                // if it's a string just keep pushing the characters
                if code_line.is_string {
                    code_line.push_char(current_char);
                    if current_char == '"' {
                        let previous_char = code_line.text.chars().last().unwrap_or(' ');
                        // end of the string
                        if previous_char != '\\' {
                            code_line.end_string();
                        }
                    }
                } else {
                    match current_char {
                        ' ' => {
                            // clean all incoming extra whitespace
                            while let Some((_, next_char)) = iter.peek() {
                                if *next_char == ' ' {
                                    iter.next();
                                } else {
                                    break;
                                }
                            }

                            if let Some((_, next_char)) = iter.peek() {
                                let next_char = *next_char;

                                match next_char {
                                    '(' | ';' | ':' => {} // do nothing, handle it in next turn
                                    _ => {
                                        // add whitespace if it is not already there
                                        code_line.append_with_whitespace("");
                                    }
                                }
                            }
                        }

                        // handle equality/assignment
                        '=' => {
                            if let Some((_, next_char)) = iter.peek() {
                                let next_char = *next_char;
                                if next_char == '=' {
                                    // it's equality operator
                                    code_line.append_with_whitespace("== ");
                                    iter.next();
                                } else if next_char == '>' {
                                    // it's fat arrow
                                    code_line.append_with_whitespace("=> ");
                                    iter.next();
                                } else {
                                    // it's assignment
                                    code_line.append_with_whitespace("= ");
                                }
                            } else {
                                code_line.append_with_whitespace("= ");
                            }
                        }

                        // handle line breakers
                        '{' => {
                            code_line.append_with_whitespace("{");
                            code_line.complete();
                            self.complete_and_add_line(code_line);
                            self.indent();

                            // if there is more -  push to new line!
                            return self.continue_to_next_line(line, iter);
                        }
                        '}' => {
                            // if there was something prior to this, move to new line
                            if !code_line.text.is_empty() {
                                self.complete_and_add_line(code_line);
                            }

                            self.outdent();

                            // clean all incoming extra whitespace
                            while let Some((_, next_char)) = iter.peek() {
                                if *next_char == ' ' {
                                    iter.next();
                                } else {
                                    break;
                                }
                            }

                            match iter.peek() {
                                Some((_, ';')) => {
                                    // check is there an ';' and add it after '}'
                                    self.complete_and_add_line(CodeLine::new("};".into()));
                                    iter.next();
                                    return self.continue_to_next_line(line, iter);
                                }
                                Some(c) => {
                                    // if there is more -  push to new line!
                                    return self.continue_to_next_line(line, iter);
                                }
                                None => {
                                    return self.complete_and_add_line(CodeLine::new("}".into()));
                                }
                            }
                        }

                        ';' => {
                            code_line.push_char(';');
                            self.handle_semicolon_case(code_line);

                            // if there is more - push to new line!
                            return self.continue_to_next_line(line, iter);
                        }

                        ':' => {
                            if let Some((_, next_char)) = iter.peek() {
                                let next_char = *next_char;
                                if next_char == ':' {
                                    // it's :: operator
                                    code_line.push_str("::");
                                    iter.next();
                                } else {
                                    code_line.push_str(": ");
                                }
                            } else {
                                code_line.push_str(": ");
                            }
                        }

                        ',' => code_line.push_str(", "),

                        // handle operators
                        '-' => {
                            if let Some((_, next_char)) = iter.peek() {
                                if *next_char == '>' {
                                    // it's a return arrow
                                    code_line.append_with_whitespace("-> ");
                                    iter.next();
                                } else {
                                    // it's just a single '-'
                                    code_line.append_with_whitespace("- ");
                                }
                            } else {
                                code_line.append_with_whitespace("- ");
                            }
                        }
                        '+' => code_line.append_with_whitespace("+ "),
                        '*' => code_line.append_with_whitespace("* "),
                        '/' => code_line.append_with_whitespace("- "),
                        '%' => code_line.append_with_whitespace("% "),
                        '^' => code_line.append_with_whitespace("^ "),
                        '!' => code_line.append_with_whitespace("!"),

                        '|' => {
                            if let Some((_, next_char)) = iter.peek() {
                                if *next_char == '|' {
                                    // it's OR operator
                                    code_line.append_with_whitespace("|| ");
                                    iter.next();
                                } else {
                                    // it's just a single '|'
                                    code_line.append_with_whitespace("| ");
                                }
                            } else {
                                code_line.append_with_whitespace("| ");
                            }
                        }

                        '&' => {
                            if let Some((_, next_char)) = iter.peek() {
                                if *next_char == '&' {
                                    // it's AND operator
                                    code_line.append_with_whitespace("&& ");
                                    iter.next();
                                } else {
                                    // it's just a single '&'
                                    code_line.append_with_whitespace("& ");
                                }
                            } else {
                                code_line.append_with_whitespace("& ");
                            }
                        }
                        '"' => {
                            if !code_line.is_string {
                                code_line.push_char(current_char);
                                code_line.become_string();
                            }
                        }
                        _ => code_line.push_char(current_char),
                    }
                }
            } else {
                break;
            }
        }

        self.add_line(code_line);
    }

    fn handle_semicolon_case(&mut self, code_line: CodeLine) {
        if code_line.text == ";" {
            if let Some(previous_code_line) = self.edits.last() {
                // case when '}' was separated from ';' by one or more new lines
                if previous_code_line.is_completed {
                    // remove empty line first
                    if !(previous_code_line.text.chars().last().unwrap_or(' ') == '}') {
                        self.edits.pop();
                    }

                    let mut updated_code_line = self.edits.pop().unwrap();
                    updated_code_line.push_char(';');
                    self.complete_and_add_line(updated_code_line);
                }
            }
        } else {
            self.complete_and_add_line(code_line);
        }
    }

    fn continue_to_next_line(&mut self, line: &str, iter: Peekable<Enumerate<Chars>>) {
        let mut iter = iter;

        if iter.peek().is_some() {
            let (next_index, _) = iter.peek().unwrap();

            let next_line = &line[*next_index..];
            self.format_and_add(next_line);
        }
    }

    fn complete_and_add_line(&mut self, code_line: CodeLine) {
        let mut code_line = code_line;
        code_line.complete();
        self.add_line(code_line);
    }

    fn add_line(&mut self, code_line: CodeLine) {
        let mut code_line = code_line;

        if code_line.text.len() == 0 {
            // don't add more than one new empty line!
            if self
                .edits
                .last()
                .unwrap_or(&CodeLine::empty_line())
                .text
                .len()
                == 0
            {
                return;
            } else {
                // push empty line
                self.edits.push(CodeLine::empty_line());
            }
        } else {
            // todo: what about strings ?
            if code_line.was_previously_stored {
                self.edits.push(code_line);
            } else {
                code_line.update_for_storage(self.get_indentation());
                self.edits.push(code_line);
            }
        }
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn outdent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn get_indentation(&self) -> String {
        let times = (self.tab_size * self.indent_level) as usize;
        " ".repeat(times)
    }

    pub fn to_text_edit(&mut self, text_lines_count: usize) -> Vec<TextEdit> {
        let line_end = if self.edits.len() > text_lines_count {
            self.edits.len()
        } else {
            text_lines_count
        };

        // add new line at the end if needed
        if let Some(code_line) = self.edits.last() {
            if !code_line.text.is_empty() {
                self.edits.push(CodeLine::empty_line())
            }
        }

        let main_edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(line_end as u32, 0)),
            new_text: self
                .edits
                .iter()
                .map(|code_line| code_line.text.clone())
                .collect::<Vec<String>>()
                .join("\n"),
        };

        vec![main_edit]
    }
}

fn is_comment(line: &str) -> bool {
    let mut chars = line.chars();
    chars.next() == Some('/') && chars.next() == Some('/')
}
