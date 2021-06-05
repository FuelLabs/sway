use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use lspower::lsp::{Position, Range, TextEdit};

use super::parse_helpers::{clean_all_incoming_whitespace, is_comment};
use super::{
    code_line::CodeLine,
    parse_helpers::{
        handle_ampersand_case, handle_assignment_case, handle_colon_case, handle_dash_case,
        handle_pipe_case, handle_string_case, handle_whitespace_case,
    },
};

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

    pub fn to_text_edit(&mut self, text_lines_count: usize) -> Vec<TextEdit> {
        let line_end = std::cmp::max(self.edits.len(), text_lines_count) as u32;

        // add new line at the end if needed
        if let Some(code_line) = self.edits.last() {
            if !code_line.is_empty() {
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

    /// formats line of code and adds it to Vec<CodeLine>
    pub fn format_and_add(&mut self, line: &str) {
        let mut code_line = self.get_unfinished_code_line_or_new();

        let line = if !code_line.is_string {
            line.trim()
        } else {
            line
        };

        // handle comment
        if is_comment(line) {
            code_line.push_str(line);
            return self.complete_and_add_line(code_line);
        }

        // handle multiline string
        if code_line.is_string {
            code_line.push_char('\n');
        }

        let mut iter = line.chars().enumerate().peekable();

        loop {
            if let Some((_, current_char)) = iter.next() {
                if code_line.is_string {
                    handle_string_case(&mut code_line, current_char);
                } else {
                    match current_char {
                        ' ' => handle_whitespace_case(&mut code_line, &mut iter),
                        '=' => handle_assignment_case(&mut code_line, &mut iter),
                        ':' => handle_colon_case(&mut code_line, &mut iter),
                        '-' => handle_dash_case(&mut code_line, &mut iter),
                        '|' => handle_pipe_case(&mut code_line, &mut iter),
                        '&' => handle_ampersand_case(&mut code_line, &mut iter),

                        ',' => code_line.push_str(", "),
                        '+' => code_line.append_with_whitespace("+ "),
                        '*' => code_line.append_with_whitespace("* "),
                        '/' => code_line.append_with_whitespace("- "),
                        '%' => code_line.append_with_whitespace("% "),
                        '^' => code_line.append_with_whitespace("^ "),
                        '!' => code_line.append_with_whitespace("!"),

                        // handle beginning of the string
                        '"' => {
                            if !code_line.is_string {
                                code_line.push_char(current_char);
                                code_line.become_string();
                            }
                        }

                        // handle line breakers ';', '{' AND '}'
                        ';' => return self.handle_semicolon_case(line, code_line, iter),

                        '{' => {
                            code_line.append_with_whitespace("{");
                            self.complete_and_add_line(code_line);
                            self.indent();

                            // if there is more - move to new line!
                            return self.move_rest_to_new_line(line, iter);
                        }

                        '}' => return self.handle_close_brace(line, code_line, iter),
                        

                        // add the rest
                        _ => code_line.push_char(current_char),
                    }
                }
            } else {
                break;
            }
        }

        self.add_line(code_line);
    }

    /// if previous line is not completed get it, otherwise start a new one
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

    fn handle_semicolon_case(&mut self, line: &str, code_line: CodeLine, iter: Peekable<Enumerate<Chars>>) {
        let mut code_line = code_line;
        code_line.push_char(';');

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

        self.move_rest_to_new_line(line, iter);
    }

    fn handle_close_brace(&mut self, line: &str, code_line: CodeLine, iter: Peekable<Enumerate<Chars>>) {
        let mut iter = iter;

        // if there was something prior to '}', add as separate line
        if !code_line.is_empty() {
            self.complete_and_add_line(code_line);
        }

        self.outdent();
        clean_all_incoming_whitespace(&mut iter);

        match iter.peek() {
            // check is there a ';' and add it after '}'
            Some((_, ';')) => {
                self.complete_and_add_line(CodeLine::new("};".into()));
                iter.next();
                self.move_rest_to_new_line(line, iter);
            }
            // if there is more - move to new line!
            Some(_) => {
                self.move_rest_to_new_line(line, iter);
            }
            None => {
                self.complete_and_add_line(CodeLine::new("}".into()));
            }
        }
    }

    fn move_rest_to_new_line(&mut self, line: &str, iter: Peekable<Enumerate<Chars>>) {
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

        if code_line.is_empty() {
            // don't add more than one new empty line!
            if self
                .edits
                .last()
                .unwrap_or(&CodeLine::empty_line())
                .is_empty()
            {
                return;
            } else {
                // push empty line
                self.edits.push(CodeLine::empty_line());
            }
        } else {
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
}
