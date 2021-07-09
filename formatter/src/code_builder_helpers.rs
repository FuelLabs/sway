use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use super::code_line::CodeLine;

pub fn is_comment(line: &str) -> bool {
    let mut chars = line.trim().chars();
    chars.next() == Some('/') && chars.next() == Some('/')
}

pub fn is_multiline_comment(line: &str) -> bool {
    let mut chars = line.trim().chars();
    chars.next() == Some('/') && chars.next() == Some('*')
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

pub fn handle_whitespace_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    clean_all_incoming_whitespace(iter);

    if let Some((_, next_char)) = iter.peek() {
        let next_char = *next_char;

        match next_char {
            '(' | ';' | ':' | ')' | ',' | '/' => {} // do nothing, handle it in next turn
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

pub fn handle_colon_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
    if let Some((_, next_char)) = iter.peek() {
        let next_char = *next_char;

        // it's :: operator
        if next_char == ':' {
            // it's initial :: from 'use' statement so append whitespace to the left
            if is_use_statement(&code_line.text) && !code_line.text.contains("::") {
                code_line.append_with_whitespace("::");
            } else {
                code_line.push_str("::");
            }
            iter.next();
        } else {
            code_line.push_str(": ");
        }
    } else {
        code_line.push_str(": ");
    }
}

pub fn handle_star_case(code_line: &mut CodeLine) {
    if is_use_statement(&code_line.text) {
        code_line.push_char('*');
    } else {
        code_line.append_with_whitespace("* ");
    }
}

pub fn handle_dash_case(code_line: &mut CodeLine, iter: &mut Peekable<Enumerate<Chars>>) {
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

pub fn clean_all_incoming_whitespace(iter: &mut Peekable<Enumerate<Chars>>) {
    while let Some((_, next_char)) = iter.peek() {
        if *next_char == ' ' {
            iter.next();
        } else {
            break;
        }
    }
}

pub fn is_dep_statement(text: &str) -> bool {
    let text = text.trim();

    match text.len() {
        3 => &text[0..3] == "dep",
        n if n > 3 => &text[0..4] == "dep ",
        _ => false,
    }
}

fn is_use_statement(text: &str) -> bool {
    let text = text.trim();

    match text.len() {
        3 => &text[0..3] == "use",
        n if n > 3 => &text[0..4] == "use ",
        _ => false,
    }
}
