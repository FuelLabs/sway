use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use crate::code_builder_helpers::{
    get_already_formatted_line_pattern, get_new_line_pattern, is_else_statement_next,
};

use super::{
    code_builder_helpers::{
        clean_all_whitespace, handle_ampersand_case, handle_assignment_case, handle_colon_case,
        handle_dash_case, handle_forward_slash_case, handle_logical_not_case,
        handle_multiline_comment_case, handle_multiply_case, handle_pipe_case, handle_plus_case,
        handle_string_case, handle_whitespace_case, is_comment, is_multiline_comment,
    },
    code_line::{CodeLine, CodeType},
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

        (num_of_lines, self.build_string())
    }

    /// formats line of code and adds it to Vec<CodeLine>
    pub fn format_and_add(&mut self, line: &str) {
        let mut code_line = self.get_unfinished_code_line_or_new(line);

        let is_string_or_multiline_comment =
            code_line.is_string() || code_line.is_multiline_comment();

        let line = if !is_string_or_multiline_comment {
            line.trim()
        } else {
            line
        };

        // add newline if it's multiline string or comment
        if is_string_or_multiline_comment {
            code_line.push_char('\n');
        } else if let Some((formatted_line, rest)) = get_already_formatted_line_pattern(line) {
            code_line.push_str(formatted_line);
            self.complete_and_add_line(code_line);
            return self.move_rest_to_new_line(rest, rest.chars().enumerate().peekable());
        }

        let mut iter = line.chars().enumerate().peekable();

        while let Some((current_index, current_char)) = iter.next() {
            match code_line.get_type() {
                CodeType::MultilineComment => {
                    handle_multiline_comment_case(&mut code_line, current_char, &mut iter);
                    if !code_line.is_multiline_comment() {
                        self.complete_and_add_line(code_line);
                        return self.move_rest_to_new_line(line, iter);
                    }
                }
                CodeType::String => handle_string_case(&mut code_line, current_char),

                _ => {
                    match current_char {
                        ' ' => handle_whitespace_case(&mut code_line, &mut iter),
                        '=' => handle_assignment_case(&mut code_line, &mut iter),
                        ':' => handle_colon_case(&mut code_line, &mut iter),
                        '-' => handle_dash_case(&mut code_line, &mut iter),
                        '|' => handle_pipe_case(&mut code_line, &mut iter),
                        '&' => handle_ampersand_case(&mut code_line, &mut iter),

                        '+' => handle_plus_case(&mut code_line, &mut iter),
                        '*' => handle_multiply_case(&mut code_line, &mut iter),
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
                                    code_line.append_with_whitespace(comment);
                                    return self.complete_and_add_line(code_line);
                                }
                                _ => handle_forward_slash_case(&mut code_line, &mut iter),
                            }
                        }
                        '%' => code_line.append_with_whitespace("% "),
                        '^' => code_line.append_with_whitespace("^ "),
                        '!' => handle_logical_not_case(&mut code_line, &mut iter),

                        // handle beginning of the string
                        '"' => {
                            if !code_line.is_string() {
                                if code_line.get_last_char() == Some('(') {
                                    code_line.push_char('"');
                                } else {
                                    code_line.append_with_whitespace("\"");
                                }
                                code_line.become_string();
                            }
                        }

                        '(' => {
                            let trimmed_text = code_line.text.trim();
                            if trimmed_text.len() >= 2 {
                                let last_two_chars = &trimmed_text[trimmed_text.len() - 2..];
                                if last_two_chars == "if" {
                                    code_line.push_char(' ');
                                }
                            }
                            code_line.push_char('(');
                        }

                        // handle line breakers ';', '{', '}' & ','
                        ',' => {
                            let rest_of_line = &line[current_index + 1..];
                            match get_new_line_pattern(rest_of_line) {
                                Some(line_after_pattern) => {
                                    code_line.push_char(',');
                                    self.complete_and_add_line(code_line);

                                    return self.move_rest_to_new_line(
                                        line_after_pattern,
                                        line_after_pattern.chars().enumerate().peekable(),
                                    );
                                }
                                None => code_line.push_str(", "),
                            }
                        }
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
                                if let Some(last_char) = code_line.get_last_char() {
                                    if last_char.is_alphabetic() && current_char.is_alphabetic() {
                                        code_line.append_whitespace()
                                    }
                                }
                            }

                            code_line.push_char(current_char)
                        }
                    }
                }
            }
        }

        self.add_line(code_line);
    }

    fn build_string(&mut self) -> String {
        self.edits
            .iter()
            .map(|code_line| code_line.text.clone())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// if previous line is not completed get it, otherwise start a new one
    fn get_unfinished_code_line_or_new(&mut self, incoming_line: &str) -> CodeLine {
        match self.edits.last() {
            Some(code_line) => {
                if code_line.is_completed {
                    // check if 'else' statement is incoming
                    if code_line.get_last_char() == Some('}')
                        && is_else_statement_next(incoming_line)
                    {
                        let mut code_line = self.edits.pop().unwrap();
                        code_line.append_whitespace();
                        code_line
                    } else {
                        CodeLine::default()
                    }
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
                    if previous_code_line.get_last_char() != Some('}') {
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

        // clean empty space before '}'
        if let Some(last_line) = self.edits.last() {
            if last_line.text.is_empty() {
                self.edits.pop();
            }
        }

        self.outdent();
        clean_all_whitespace(&mut iter);

        match iter.peek() {
            // check is there a ';' and add it after '}'
            Some((_, ';')) => {
                self.complete_and_add_line(CodeLine::new("};".into()));
                iter.next();
                self.move_rest_to_new_line(line, iter);
            }
            Some((_, ',')) => {
                self.complete_and_add_line(CodeLine::new("},".into()));
                iter.next();
                self.move_rest_to_new_line(line, iter);
            }
            // if there is more move to new line, unless it's 'else' statement or ')' | '{'
            Some((next_index, next_char)) => {
                let next_line = &line[*next_index..].trim();
                let is_valid_char = *next_char == '{' || *next_char == ')';

                if is_valid_char {
                    self.add_line(CodeLine::new("}".into()));
                    self.format_and_add(next_line);
                } else if is_else_statement_next(next_line) {
                    self.add_line(CodeLine::new("} ".into()));
                    self.format_and_add(next_line);
                } else {
                    self.complete_and_add_line(CodeLine::new("}".into()));
                    self.move_rest_to_new_line(line, iter);
                }
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
            if is_comment(next_line) {
                if let Some(mut code_line) = self.edits.pop() {
                    code_line.push_char(' ');
                    code_line.push_str(next_line);
                    self.add_line(code_line);
                }
            } else if is_multiline_comment(next_line) {
                if let Some(mut code_line) = self.edits.pop() {
                    code_line.is_completed = false;
                    code_line.push_char(' ');
                    self.add_line(code_line);
                    self.format_and_add(next_line);
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
            if !self
                .edits
                .last()
                .unwrap_or(&CodeLine::empty_line())
                .is_empty()
            {
                // only add empty line if previous last char wasn't '{'
                if !self.edits.last().unwrap().text.ends_with('{') {
                    self.edits.push(CodeLine::empty_line());
                }
            }
        } else if code_line.was_previously_stored {
            self.edits.push(code_line);
        } else {
            code_line.update_for_storage(self.get_indentation());
            self.edits.push(code_line);
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
