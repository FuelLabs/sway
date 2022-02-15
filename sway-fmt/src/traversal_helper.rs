use super::code_builder_helpers::{is_comment, is_multiline_comment, is_newline_incoming};
use crate::code_builder_helpers::clean_all_whitespace;
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, NEW_LINE_PATTERN};
use std::iter::{Enumerate, Peekable};
use std::slice::Iter;
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

/// Trims whitespaces and reorders compound import statements lexicographically
/// a::{c, b, d::{self, f, e}} -> a::{b,c,d::{self,e,f}}
fn sort_and_filter_use_expression(line: &str) -> String {
    /// Tokenizes the line on separators keeping the separators.
    fn tokenize(line: &str) -> Vec<String> {
        let mut buffer: Vec<String> = Vec::new();
        let mut current = 0;
        for (index, separator) in line.match_indices(|c: char| c == ',' || c == '{' || c == '}') {
            if index != current {
                buffer.push(line[current..index].to_string());
            }
            buffer.push(separator.to_string());
            current = index + separator.len();
        }
        if current < line.len() {
            buffer.push(line[current..].to_string());
        }
        buffer
    }
    let tokens: Vec<String> = tokenize(line);
    let mut buffer: Vec<String> = Vec::new();

    fn sort_imports(tokens: &mut Iter<String>, buffer: &mut Vec<String>) {
        let token = tokens.next();
        match token.map(|t| t.trim()) {
            None => return,
            Some(",") => (),
            Some("{") => {
                let mut inner_buffer: Vec<String> = Vec::new();
                sort_imports(tokens, &mut inner_buffer);
                if !inner_buffer.is_empty() {
                    if let Some(buff) = buffer.last_mut() {
                        buff.push_str(inner_buffer[0].as_str());
                    } else {
                        buffer.append(&mut inner_buffer);
                    }
                }
            }
            Some("}") => {
                buffer.sort_by(|a, b| {
                    if *a == "self" {
                        std::cmp::Ordering::Less
                    } else if *b == "self" {
                        std::cmp::Ordering::Greater
                    } else {
                        a.cmp(b)
                    }
                });
                if buffer.len() > 1 {
                    *buffer = vec![format!("{{{}}}", buffer.join(", "))];
                }
                return;
            }
            Some(c) => buffer.push(c.to_string()),
        }
        sort_imports(tokens, buffer);
    }
    sort_imports(&mut tokens.iter(), &mut buffer);
    buffer.concat()
}

pub fn format_use_statement(line: &str) -> String {
    let use_keyword = extract_keyword(line, Rule::use_keyword).unwrap();
    let (_, right) = line.split_once(&use_keyword).unwrap();
    let right: String = sort_and_filter_use_expression(right);
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

#[cfg(test)]
mod tests {
    use super::sort_and_filter_use_expression;

    #[test]
    fn test_sort_and_filter_use_expression() {
        assert_eq!(sort_and_filter_use_expression("::a::b::c;"), "::a::b::c;");
        assert_eq!(
            sort_and_filter_use_expression("::a::c::b::{c, b, ba};"),
            "::a::c::b::{b, ba, c};"
        );
        assert_eq!(
            sort_and_filter_use_expression("{s,e,l,f,self};"),
            "{self, e, f, l, s};"
        );
        assert_eq!(
            sort_and_filter_use_expression("a::{d::{f, self}, c, b};"),
            "a::{b, c, d::{self, f}};"
        );
        assert_eq!(
            sort_and_filter_use_expression("a::b::{c,d::{self,f}};"),
            "a::b::{c, d::{self, f}};"
        );
        assert_eq!(sort_and_filter_use_expression("a::b::{c};"), "a::b::c;");
        assert_eq!(
            sort_and_filter_use_expression("a::b::{c,d::{e}};"),
            "a::b::{c, d::e};"
        );
    }
}
