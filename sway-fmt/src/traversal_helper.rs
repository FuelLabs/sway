use super::code_builder_helpers::{is_comment, is_multiline_comment, is_newline_incoming};
use crate::code_builder_helpers::clean_all_whitespace;
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, NEW_LINE_PATTERN};
use std::iter::{Enumerate, Peekable};
use std::slice::Iter;
use std::{fmt::Write, str::Chars};

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

// Same as s.match_indices(|ch| {}) but allows to match by checking &str vs char
fn match_indices_str(s: &str) -> Vec<(usize, &str)> {
    let mut res: Vec<(usize, &str)> = Vec::new();

    // Match the as token with spaces so as to avoid imprperly matching an 'as' substring
    // in another type of token
    let as_token = " as ";
    let mut start = 0;

    while start < s.len() {
        // Try to match the 'as' token first then fallback to single chars
        if start <= s.len() - as_token.len()
            && s.len() >= as_token.len()
            && &s[start..start + as_token.len()] == as_token
        {
            res.push((start + 1, as_token.trim()));
            start += as_token.len();
            continue;
        }

        match &s[start..start + 1] {
            "," => {
                res.push((start, ","));
            }
            "{" => {
                res.push((start, "{"));
            }
            "}" => {
                res.push((start, "}"));
            }
            _ => {}
        };

        start += 1;
    }

    res
}

/// Tokenizes the line on separators keeping the separators.
fn tokenize(line: &str) -> Vec<String> {
    let mut buffer: Vec<String> = Vec::new();
    let mut current = 0;
    for (index, separator) in match_indices_str(line) {
        if index != current {
            // Chomp all whitespace including newlines, and only push
            // resulting token if what's left is not an empty string. This
            // is needed to ignore trailing commas with newlines.
            let to_push: String = line[current..index]
                .to_string()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();

            if !to_push.is_empty() {
                buffer.push(to_push);
            }
        }
        buffer.push(separator.to_string());
        current = index + separator.len();
    }
    if current < line.len() {
        buffer.push(line[current..].to_string());
    }
    buffer
}

/// Trims whitespaces and reorders compound import statements lexicographically
/// a::{c, b, d::{self, f, e}} -> a::{b,c,d::{self,e,f}}
fn sort_and_filter_use_expression(line: &str) -> String {
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
            Some("as") => {
                // There should always be a name before an 'as' token
                let prev = buffer.pop().unwrap();
                let alias = tokens.next().unwrap();
                buffer.push(format!("{} {} {}", prev, "as", alias));
            }
            Some(c) => buffer.push(c.to_string()),
        }
        sort_imports(tokens, buffer);
    }
    sort_imports(&mut tokens.iter(), &mut buffer);
    buffer.concat()
}

fn format_use_statement_length(s: &str, max_length: usize, level: usize) -> String {
    let s = match s.starts_with(ALREADY_FORMATTED_LINE_PATTERN) {
        true => s[ALREADY_FORMATTED_LINE_PATTERN.len()..].trim(),
        false => s,
    };

    let buff = tokenize(s);
    let mut without_newline = buff.iter().rev().collect::<Vec<&String>>();

    let len: usize = buff.iter().map(|x| x.len()).sum();
    if len <= max_length {
        return s.to_owned();
    }

    // Receive tokens and push them to a string until a full line is made
    fn make_line(token: &str, line: &mut String, open_brackets: &mut u8, remainder: usize) -> bool {
        let mut is_line = false;

        match token {
            "," => {
                let _ = write!(line, "{} ", token).map_err(|_| ());
                if *open_brackets == 1 {
                    is_line = true;
                }
            }
            "{" => {
                line.push_str(token);
                if *open_brackets == 0 {
                    is_line = true;
                }
                *open_brackets += 1;
            }
            "}" => {
                line.push_str(token);
                *open_brackets -= 1;
                // Using `remainder` to see if we're at either a 2-char terminator for the full
                // use statement (i.e., '};') or at a single char terminator (e.g., '}') for individual
                // formatted lines
                if *open_brackets == 1 && (remainder == 2 || remainder == 1) {
                    is_line = true;
                }
            }
            "as" => {
                let _ = write!(line, " {} ", token);
            }
            _ => {
                line.push_str(token);
                if remainder == 2 && *open_brackets == 1 {
                    line.push(',');
                    is_line = true;
                }
            }
        }

        is_line
    }

    fn format_line(input: &str, open_brackets: u8, level: usize) -> String {
        let input = input.trim();
        let mut tabs = open_brackets as usize + level;

        let mut output = match input.starts_with(ALREADY_FORMATTED_LINE_PATTERN) {
            true => input.to_owned(),
            false => ALREADY_FORMATTED_LINE_PATTERN.to_owned(),
        };

        // If this is the end of nested brackets, decrement `tabs` if we have any
        if (input.ends_with('{') || input.ends_with("};") || open_brackets > 1) && tabs > 0 {
            tabs -= 1;
        }

        let prefix = "    ".repeat(tabs);
        output.push_str(&prefix);
        output.push_str(input);

        if tabs > 0 || input.ends_with('{') || input.ends_with("};") {
            output.push('\n');
        }

        output
    }

    let mut with_newline: Vec<String> = Vec::new();

    let mut curr_line = String::new();
    let mut open_brackets = 0u8;

    while let Some(token) = without_newline.pop() {
        let is_line = make_line(
            token,
            &mut curr_line,
            &mut open_brackets,
            without_newline.len(),
        );

        if !is_line {
            continue;
        }

        curr_line = format_line(&curr_line, open_brackets, level);

        if curr_line.len() > max_length {
            curr_line = format_use_statement_length(&curr_line, max_length, level + 1);
        }

        with_newline.push(curr_line);
        curr_line = String::new();
    }

    if !curr_line.is_empty() {
        curr_line = format_line(&curr_line, open_brackets, level);
        with_newline.push(curr_line);
    }

    with_newline.concat()
}

// this will be replaced in v2 anyway
pub fn format_use_statement(line: &str) -> String {
    let mut line = line.trim().split(' ');
    let use_keyword = line
        .next()
        .expect("err: format_use_statement called on non-use-statement");
    let line = line.collect::<Vec<&str>>().join(" ");
    let mut line: String = sort_and_filter_use_expression(&line);

    let max_length = 100usize;

    // This is mostly to satisfy a failing fmt test
    if line.len() > max_length {
        line = format_use_statement_length(&line, max_length, 0usize);
        line.insert_str(
            ALREADY_FORMATTED_LINE_PATTERN.len(),
            &format!("{} ", use_keyword),
        );
    } else {
        line = format!("{}{} {}", ALREADY_FORMATTED_LINE_PATTERN, use_keyword, line)
    }

    line
}

pub fn format_include_statement(line: &str) -> String {
    let mut line = line.trim().split(' ');
    let include_keyword = line
        .next()
        .expect("err: format_include_statement called on non-include-statement");
    let line = line.collect::<Vec<&str>>().join(" ");
    let line: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    format!(
        "{}{} {}",
        ALREADY_FORMATTED_LINE_PATTERN, include_keyword, line
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
    use super::{format_use_statement_length, sort_and_filter_use_expression};
    use crate::constants::ALREADY_FORMATTED_LINE_PATTERN;

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
        assert_eq!(
            sort_and_filter_use_expression("a::{foo,bar,};"),
            "a::{bar, foo};"
        );
        assert_eq!(
            sort_and_filter_use_expression(
                "a::{
    foo,
    bar,
};"
            ),
            "a::{bar, foo};"
        );
    }

    #[test]
    fn test_format_use_statement_length_leaves_input_unchanged() {
        let s = "a::b::{c, d::{self, f}};";
        assert_eq!(format_use_statement_length(s, 100, 0), s);
    }

    #[test]
    fn test_format_use_statement_length_formats_long_input() {
        let s = "std::{address::*, assert::assert, block::*, chain::auth::*, context::{*,text::{call_frames::*, dial_frames::{Transaction, TransactionParameters}, token_storage::{CallData, Parameters}}}, contract_id::ContractId, hash::*, panic::panic, storage::*, token::*};";
        let expected = format!(
            r#"{ALREADY_FORMATTED_LINE_PATTERN}std::{{
{ALREADY_FORMATTED_LINE_PATTERN}    address::*,
{ALREADY_FORMATTED_LINE_PATTERN}    assert::assert,
{ALREADY_FORMATTED_LINE_PATTERN}    block::*,
{ALREADY_FORMATTED_LINE_PATTERN}    chain::auth::*,
{ALREADY_FORMATTED_LINE_PATTERN}    context::{{
{ALREADY_FORMATTED_LINE_PATTERN}        *,
{ALREADY_FORMATTED_LINE_PATTERN}        text::{{
{ALREADY_FORMATTED_LINE_PATTERN}            call_frames::*,
{ALREADY_FORMATTED_LINE_PATTERN}            dial_frames::{{Transaction, TransactionParameters}},
{ALREADY_FORMATTED_LINE_PATTERN}            token_storage::{{CallData, Parameters}}
{ALREADY_FORMATTED_LINE_PATTERN}        }}
{ALREADY_FORMATTED_LINE_PATTERN}    }},
{ALREADY_FORMATTED_LINE_PATTERN}    contract_id::ContractId,
{ALREADY_FORMATTED_LINE_PATTERN}    hash::*,
{ALREADY_FORMATTED_LINE_PATTERN}    panic::panic,
{ALREADY_FORMATTED_LINE_PATTERN}    storage::*,
{ALREADY_FORMATTED_LINE_PATTERN}    token::*,
{ALREADY_FORMATTED_LINE_PATTERN}}};
"#
        );
        assert_eq!(format_use_statement_length(s, 100, 0), expected);
    }

    #[test]
    fn test_format_use_statement_formats_long_input_with_aliases() {
        let s = "std::{address::*, assert::assert as LocalAssert, block::*, chain::auth::*, context::{*,text::{call_frames::*, dial_frames::{Transaction as DialFrameTransaction, TransactionParameters}, token_storage::{CallData, Parameters}}}, contract_id::ContractId, hash::*, panic::panic, storage::*, token::*};";
        let expected = format!(
            r#"{ALREADY_FORMATTED_LINE_PATTERN}std::{{
{ALREADY_FORMATTED_LINE_PATTERN}    address::*,
{ALREADY_FORMATTED_LINE_PATTERN}    assert::assert as LocalAssert,
{ALREADY_FORMATTED_LINE_PATTERN}    block::*,
{ALREADY_FORMATTED_LINE_PATTERN}    chain::auth::*,
{ALREADY_FORMATTED_LINE_PATTERN}    context::{{
{ALREADY_FORMATTED_LINE_PATTERN}        *,
{ALREADY_FORMATTED_LINE_PATTERN}        text::{{
{ALREADY_FORMATTED_LINE_PATTERN}            call_frames::*,
{ALREADY_FORMATTED_LINE_PATTERN}            dial_frames::{{Transaction as DialFrameTransaction, TransactionParameters}},
{ALREADY_FORMATTED_LINE_PATTERN}            token_storage::{{CallData, Parameters}}
{ALREADY_FORMATTED_LINE_PATTERN}        }}
{ALREADY_FORMATTED_LINE_PATTERN}    }},
{ALREADY_FORMATTED_LINE_PATTERN}    contract_id::ContractId,
{ALREADY_FORMATTED_LINE_PATTERN}    hash::*,
{ALREADY_FORMATTED_LINE_PATTERN}    panic::panic,
{ALREADY_FORMATTED_LINE_PATTERN}    storage::*,
{ALREADY_FORMATTED_LINE_PATTERN}    token::*,
{ALREADY_FORMATTED_LINE_PATTERN}}};
"#
        );
        assert_eq!(format_use_statement_length(s, 100, 0), expected);
    }
}
