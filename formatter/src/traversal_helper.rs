use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use core_lang::{extract_keyword, Rule};

use super::code_builder_helpers::{is_comment, is_multiline_comment};
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, NEW_LINE_PATTERN};

pub fn format_struct(text: &str) -> String {
    let mut iter = text.chars().enumerate().peekable();

    let mut is_curr_comment = false;
    let mut is_curr_multi_comment = false;

    let mut result = String::default();

    loop {
        if let Some((_, current_char)) = iter.next() {
            if is_curr_comment {
                result.push(current_char);
                if current_char == '\n' {
                    is_curr_comment = false;
                }
            } else if is_curr_multi_comment {
                result.push(current_char);
                if current_char == '*' {
                    if let Some((_, c)) = iter.peek() {
                        if *c == '/' {
                            iter.next();
                            result.push('/');
                            is_curr_multi_comment = false;
                        }
                    }
                }
            } else {
                if current_char != '\n' && current_char != ',' {
                    result.push(current_char);
                }

                match current_char {
                    '/' => match iter.peek() {
                        Some((_, '/')) => {
                            result.push('/');
                            iter.next();
                            is_curr_comment = true;
                        }
                        Some((_, '*')) => {
                            result.push('*');
                            iter.next();
                            is_curr_multi_comment = true;
                        }
                        _ => {}
                    },
                    ':' => {
                        let struct_field_type = get_struct_field_type(text, &mut iter);
                        result.push_str(&struct_field_type);
                    }
                    _ => {}
                }
            }
        } else {
            break;
        }
    }

    result
}

pub fn format_use_statement(line: &str) -> String {
    let use_keyword = extract_keyword(line, Rule::use_keyword).unwrap();
    let (_, right) = line.split_once(use_keyword).unwrap();
    let right: String = right.chars().filter(|c| !c.is_whitespace()).collect();
    format!(
        "{}{} {}",
        ALREADY_FORMATTED_LINE_PATTERN, use_keyword, right
    )
}

pub fn format_include_statement(line: &str) -> String {
    let include_keyword = extract_keyword(line, Rule::include_keyword).unwrap();
    let (_, right) = line.split_once(include_keyword).unwrap();
    let right: String = right.chars().filter(|c| !c.is_whitespace()).collect();
    format!(
        "{}{} {}",
        ALREADY_FORMATTED_LINE_PATTERN, include_keyword, right
    )
}

fn get_struct_field_type(line: &str, iter: &mut Peekable<Enumerate<Chars>>) -> String {
    let mut result = String::default();

    loop {
        match iter.peek() {
            Some((_, c)) => {
                let next_char = *c;

                match next_char {
                    ',' => {
                        iter.next();
                        result.push(',');
                        break;
                    }
                    '\n' | '}' | '/' => {
                        result.push(',');
                        break;
                    }
                    _ => {
                        iter.next();
                        result.push(next_char);
                    }
                }
            }

            None => {
                result.push(',');
                break;
            }
        }
    }

    if let Some((next_index, _)) = iter.peek() {
        let leftover = &line[*next_index..];
        if !is_comment(leftover) && !is_multiline_comment(leftover) {
            result.push_str(NEW_LINE_PATTERN);
        }
    }

    result
}
