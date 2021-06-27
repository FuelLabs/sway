use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use super::{
    code_line::CodeLine,
    parse_helpers::{
        clean_all_incoming_whitespace, handle_ampersand_case, handle_assignment_case,
        handle_colon_case, handle_dash_case, handle_multiline_comment_case, handle_pipe_case,
        handle_string_case, handle_whitespace_case, is_comment,
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

    pub fn get_final_edits(mut self) -> (usize, String) {
        // add new line at the end if needed
        if let Some(code_line) = self.edits.last() {
            if !code_line.is_empty() {
                self.edits.push(CodeLine::empty_line())
            }
        }

        let num_of_lines = self.edits.len();

        (num_of_lines, self.to_string())
    }

    /// formats line of code and adds it to Vec<CodeLine>
    pub fn format_and_add(&mut self, line: &str) {
        let mut code_line = self.get_unfinished_code_line_or_new();

        let is_string_or_multiline_comment =
            code_line.is_string() || code_line.is_multiline_comment();

        let line = if !is_string_or_multiline_comment {
            line.trim()
        } else {
            line
        };

        // handle comment
        if is_comment(line) {
            code_line.push_str(line);
            return self.complete_and_add_line(code_line);
        }

        // add newline if it's multiline string or comment
        if is_string_or_multiline_comment {
            code_line.push_char('\n');
        }

        if code_line.is_multiline_comment() && line.trim() == "*/" {
            code_line.push_str(&self.get_indentation());
            code_line.push_str("*/");
            return self.complete_and_add_line(code_line);
        }

        let mut iter = line.chars().enumerate().peekable();

        loop {
            if let Some((current_index, current_char)) = iter.next() {
                if code_line.is_string() {
                    handle_string_case(&mut code_line, current_char);
                } else if code_line.is_multiline_comment() {
                    handle_multiline_comment_case(&mut code_line, current_char, &mut iter);
                    if !code_line.is_multiline_comment() {
                        self.complete_and_add_line(code_line);
                        return self.move_rest_to_new_line(line, iter);
                    }
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
                        '/' => {
                            match iter.peek() {
                                Some((_, '*')) => {
                                    // it's a multiline comment
                                    code_line.become_multiline_comment();
                                    iter.next();
                                    code_line.push_str("/*");
                                }
                                Some((_, '/')) => {
                                    // it's a comment
                                    let comment = &line[current_index..];
                                    code_line.append_with_whitespace(&comment);
                                    return self.complete_and_add_line(code_line);
                                }
                                _ => code_line.append_with_whitespace("/ "),
                            }
                        }
                        '%' => code_line.append_with_whitespace("% "),
                        '^' => code_line.append_with_whitespace("^ "),
                        '!' => code_line.append_with_whitespace("!"),

                        // handle beginning of the string
                        '"' => {
                            if !code_line.is_string() {
                                code_line.append_with_whitespace("\"");
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
                        _ => {
                            // handle case when keywords are on different lines
                            if current_index == 0 {
                                // if there are 2 keywords on different lines - add whitespace between them
                                if let Some(last_char) = code_line.text.chars().last() {
                                    if last_char.is_alphabetic() && current_char.is_alphabetic() {
                                        code_line.append_whitespace()
                                    }
                                }
                            }

                            code_line.push_char(current_char)
                        }
                    }
                }
            } else {
                break;
            }
        }

        self.add_line(code_line);
    }

    fn to_string(&mut self) -> String {
        self.edits
            .iter()
            .map(|code_line| code_line.text.clone())
            .collect::<Vec<String>>()
            .join("\n")
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

    fn handle_semicolon_case(
        &mut self,
        line: &str,
        code_line: CodeLine,
        iter: Peekable<Enumerate<Chars>>,
    ) {
        let mut code_line = code_line;
        code_line.push_char(';');

        if code_line.text == ";" {
            if let Some(previous_code_line) = self.edits.last() {
                // case when '}' was separated from ';' by one or more new lines
                if previous_code_line.is_completed {
                    // remove empty line first
                    if !(previous_code_line.text.chars().last() == Some('}')) {
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

    fn handle_close_brace(
        &mut self,
        line: &str,
        code_line: CodeLine,
        iter: Peekable<Enumerate<Chars>>,
    ) {
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
                self.complete_and_add_line(CodeLine::new("}".into()));
                self.move_rest_to_new_line(line, iter);
            }
            None => {
                self.complete_and_add_line(CodeLine::new("}".into()));
            }
        }
    }

    fn move_rest_to_new_line(&mut self, line: &str, iter: Peekable<Enumerate<Chars>>) {
        let mut iter = iter;

        if let Some((next_index, _)) = iter.peek() {
            let next_line = &line[*next_index..].trim();

            // if rest is comment append it to the last existing line
            if is_comment(&next_line) {
                if let Some(mut code_line) = self.edits.pop() {
                    code_line.push_char(' ');
                    code_line.push_str(next_line);
                    self.add_line(code_line);
                }
            } else {
                self.format_and_add(next_line);
            }
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
