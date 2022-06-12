use super::code_line::CodeLine;
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, NEW_LINE_PATTERN};
use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

pub fn is_comment(line: &str) -> bool {
    let mut chars = line.trim().chars();
    chars.next() == Some('/') && chars.next() == Some('/')
}

pub fn is_else_statement_next(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 4 && &trimmed[0..4] == "else"
}

pub fn is_multiline_comment(line: &str) -> bool {
    let mut chars = line.trim().chars();
    chars.next() == Some('/') && chars.next() == Some('*')
}

/// checks for newline only, ignores an empty space
pub fn is_newline_incoming(line: &str) -> bool {
    let chars = line.chars();

    for c in chars {
        match c {
            '\n' => return true,
            ' ' => {}
            _ => return false,
        }
    }

    false
}

pub fn handle_multiline_comment_case(
    code_line: &mut CodeLine,
    current_char: char,
    iter: &mut Peekable<Enumerate<Chars>>,
) {
    code_line.push_char(current_char);

    if current_char == '*' {
        // end multiline comment and reset to default type
        if let Some((_, '/')) = iter.peek() {
            code_line.push_char('/');
            iter.next();
            code_line.become_default();
        }
    }
}

// if it's a string just keep pushing the characters
pub fn handle_string_case(code_line: &mut CodeLine, current_char: char) {
    code_line.push_char(current_char);
    if current_char == '"' {
        let previous_char = code_line.text.chars().last();
        // end of the string
        if previous_char != Some('\\') {
            code_line.become_default();
        }
    }
}

pub fn handle_logical_not_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    code_line.push_char('!');
    clean_all_whitespace(iter);
}

pub fn handle_whitespace_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    clean_all_whitespace(iter);

    if let Some((_, next_char)) = iter.peek() {
        let next_char = *next_char;

        match next_char {
            '(' | ';' | ':' | ')' | ',' | '}' => {} // do nothing, handle it in next turn
            _ => {
                // add whitespace if it is not already there
                code_line.append_whitespace();
            }
        }
    }
}

pub fn handle_assignment_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
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
            code_line.append_equal_sign();
        }
    } else {
        code_line.append_with_whitespace("= ");
    }
}

pub fn handle_plus_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    if let Some((_, next_char)) = iter.peek() {
        let next_char = *next_char;
        if next_char == '=' {
            // it's a += operator
            code_line.append_with_whitespace("+= ");
            iter.next();
        } else {
            code_line.append_with_whitespace("+ ");
        }
    } else {
        code_line.append_with_whitespace("+ ");
    }
}

pub fn handle_colon_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
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

pub fn handle_dash_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    if let Some((_, next_char)) = iter.peek() {
        if *next_char == '>' {
            // it's a return arrow
            code_line.append_with_whitespace("-> ");
            iter.next();
        } else if *next_char == '=' {
            // it's a -= operator
            code_line.append_with_whitespace("-= ");
            iter.next();
        } else {
            // it's just a single '-'
            code_line.append_with_whitespace("- ");
        }
    } else {
        code_line.append_with_whitespace("- ");
    }
}

pub fn handle_multiply_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    if let Some((_, next_char)) = iter.peek() {
        let next_char = *next_char;
        if next_char == '=' {
            // it's a *= operator
            code_line.append_with_whitespace("*= ");
            iter.next();
        } else {
            code_line.append_with_whitespace("* ");
        }
    } else {
        code_line.append_with_whitespace("* ");
    }
}

pub fn handle_pipe_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
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

pub fn handle_forward_slash_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    // Handles non-comment related /.
    if let Some((_, next_char)) = iter.peek() {
        let next_char = *next_char;
        if next_char == '=' {
            // it's a /= operator
            code_line.append_with_whitespace("/= ");
            iter.next();
        } else {
            code_line.append_with_whitespace("/ ");
        }
    } else {
        code_line.append_with_whitespace("/ ");
    }
}

pub fn handle_ampersand_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
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

/// cleans whitespace, including newlines
pub fn clean_all_whitespace(iter: &mut Peekable<Enumerate<Chars>>) {
    while let Some((_, next_char)) = iter.peek() {
        if next_char.is_whitespace() {
            iter.next();
        } else {
            break;
        }
    }
}

/// checks does next part of the line contain "add new line" pattern,
/// if it does it returns the rest of the line
pub fn get_new_line_pattern(line: &str) -> Option<&str> {
    let pattern_len = NEW_LINE_PATTERN.len();

    if line.len() >= pattern_len && &line[0..pattern_len] == NEW_LINE_PATTERN {
        return Some(&line[pattern_len..]);
    }

    None
}

/// checks does beginning of the new line contain "already formatted" pattern
/// if it does it splits and returns the already formatted line and the rest after it
pub fn get_already_formatted_line_pattern(line: &str) -> Option<(&str, &str)> {
    let pattern_len = ALREADY_FORMATTED_LINE_PATTERN.len();

    if line.starts_with(ALREADY_FORMATTED_LINE_PATTERN) {
        let char_idxs = vec![
            line.find(';').unwrap_or(0),
            line.rfind(',').unwrap_or(0),
            line.rfind('}').unwrap_or(0),
            line.rfind('{').unwrap_or(0),
        ];

        let end = char_idxs.iter().max().unwrap();

        let formatted_line = &line[pattern_len..end + 1];
        // rest, if any
        let rest = &line[end + 1..];

        return Some((formatted_line, rest));
    }

    None
}
