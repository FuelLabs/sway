use super::code_builder_helpers::{is_comment, is_multiline_comment, is_newline_incoming};
use crate::code_builder_helpers::clean_all_whitespace;
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, NEW_LINE_PATTERN};
use std::iter::{Enumerate, Peekable};
use std::str::Chars;
use sway_core::{extract_keyword, Rule};

/// Performs the formatting of the `comments` section in your code.
/// Takes in a function that provides the logic to handle the rest of the code.
fn custom_format_with_comments<F>(text: &str, custom_format_fn: &mut F) -> String
where
    F: FnMut(&str, &mut String, char, &mut Peekable<Enumerate<Chars>>),
{
    let mut iter = text.chars().enumerate().peekable();

    let mut is_curr_comment = false;
    let mut is_curr_multi_comment = false;
    let mut result = String::default();

    while let Some((_, current_char)) = iter.next() {
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
            match current_char {
                '/' => match iter.peek() {
                    Some((_, '/')) => {
                        result.push_str("//");
                        iter.next();
                        is_curr_comment = true;
                    }
                    Some((_, '*')) => {
                        result.push_str("/*");
                        iter.next();
                        is_curr_multi_comment = true;
                    }
                    _ => custom_format_fn(text, &mut result, current_char, &mut iter),
                },
                _ => custom_format_fn(text, &mut result, current_char, &mut iter),
            }
        }
    }

    result
}

/// Formats Sway data types: Enums and Structs.
pub fn format_data_types(text: &str) -> String {
    custom_format_with_comments(text, &mut move |text, result, current_char, iter| {
        result.push(current_char);
        match current_char {
            '}' => {
                clean_all_whitespace(iter);
                if let Some((_, next_char)) = iter.peek() {
                    if *next_char != ',' {
                        result.push(',');
                    }
                }
            }
            ':' => {
                let field_type = get_data_field_type(text, iter);
                result.push_str(&field_type);
            }
            _ => {}
        }
    })
}

pub fn format_delineated_path(line: &str) -> String {
    // currently just clean up extra unwanted whitespace
    line.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn format_use_statement(line: &str) -> String {
    let use_keyword = extract_keyword(line, Rule::use_keyword).unwrap();
    let (_, right) = line.split_once(&use_keyword).unwrap();
    let right: String = right.chars().filter(|c| !c.is_whitespace()).collect();
    format!(
        "{}{} {}",
        ALREADY_FORMATTED_LINE_PATTERN, use_keyword, right
    )
}

pub fn format_include_statement(line: &str) -> String {
    let include_keyword = extract_keyword(line, Rule::include_keyword).unwrap();
    let (_, right) = line.split_once(&include_keyword).unwrap();
    let right: String = right.chars().filter(|c| !c.is_whitespace()).collect();
    format!(
        "{}{} {}",
        ALREADY_FORMATTED_LINE_PATTERN, include_keyword, right
    )
}

fn get_data_field_type(line: &str, iter: &mut Peekable<Enumerate<Chars>>) -> String {
    let mut result = String::default();

    loop {
        match iter.peek() {
            Some((next_index, c)) => {
                let next_char = *c;
                let next_index = *next_index;

                match next_char {
                    ',' => {
                        iter.next();
                        result.push(',');
                        break;
                    }
                    '{' => {
                        iter.next();
                        result.push('{');
                        return result;
                    }
                    '}' => {
                        result.push(',');
                        break;
                    }
                    '/' => {
                        let leftover = &line[next_index..next_index + 2];
                        if leftover == "//" || leftover == "/*" {
                            result.push(',');
                            break;
                        } else {
                            iter.next();
                            result.push('/');
                        }
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
        if is_newline_incoming(leftover)
            || !(is_comment(leftover) || is_multiline_comment(leftover))
        {
            result.push_str(NEW_LINE_PATTERN);
        }
    }

    result
}
