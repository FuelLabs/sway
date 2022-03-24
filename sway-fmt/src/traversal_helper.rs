use crate::code_builder_helpers::*;
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, TAB_SIZE};
use std::iter::{Enumerate, Peekable};
use std::slice::Iter;
use std::str::Chars;
use std::thread::current;
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
                clean_all_whitespace_enumerated(iter);
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

#[test]
// write test for format_data_types & custom_format_with_comments
fn test_align_struct_with_gnarly_comments() {
    let unformatted_struct = r"
    struct /* i am about to declare a struct */ DummyStruct {
        // properly handling comments?
sumn /* hi i am  a comment */ : value /* another comment */,
    sumnelse: u32, // so many comments
nocomments: u64,
    /* hi */    } // oops a comment
    ";
    let formatted = r"struct /* i am about to declare a struct */ DummyStruct {
    // properly handling comments?
    sumn /* hi i am a comment */ : value, /* another comment */
    sumnelse                      : u32, // so many comments
    nocomments                    : u64,
    /* hi */
} // oops a comment
";
    let post_formatting = _format_align_data_types(unformatted_struct);
    println!("{}", post_formatting);
    assert_eq!(post_formatting, formatted);
}

/// Formats Sway data types and aligns fields for Enums and Structs
pub fn _format_align_data_types(text: &str) -> String {
    let longest_var = find_longest_variant(text);
    let mut current_column = 0;
    let mut iter = text.chars().enumerate().peekable();
    let mut result = String::new();
    let newline_and_tab = format!("\n{}", {
        let buf = vec![" "; TAB_SIZE];
        buf.join("")
    });
    clean_all_whitespace_enumerated(&mut iter);

    while let Some((_, current_char)) = iter.next() {
        match dbg!(current_char) {
            '}' => {
                clean_all_whitespace_enumerated(&mut iter);
                if current_column != 0 {
                    result.push('\n');
                }
                result.push(current_char);
                result.push(' ');
                current_column = 0;
            }
            ':' => {
                clean_all_whitespace_enumerated(&mut iter);
                let field_type = get_data_field_type(text, &mut iter);
                while current_column < longest_var {
                    result.push(' ');
                    current_column += 1;
                }
                result.push(current_char);
                result.push(' ');
                result.push_str(&field_type);
                if iter.peek() != Some(&(current_column, ' '))
                    || iter.peek() != Some(&(current_column, '\n'))
                {
                    if let Some((_, '/')) = iter.peek() {
                        clean_all_whitespace_enumerated(&mut iter);
                        result.push(' ');
                    }
                }
            }
            ',' => (),
            '{' => {
                current_column = 0;
                result.push(current_char);
                result.push_str(&newline_and_tab);
                clean_all_whitespace_enumerated(&mut iter);
            }
            '\n' => {
                clean_all_whitespace_enumerated(&mut iter);
                current_column = 0;
                if let Some((_, _c)) = iter.peek() {
                    result.push_str(&newline_and_tab);
                } else {
                    result.push('\n');
                }
            }
            ' ' => {
                if let Some((_, ' ')) = iter.peek() {
                    clean_all_whitespace_enumerated(&mut iter);
                    result.push(' ');
                    current_column += 1;
                } else {
                    result.push(' ');
                }
                current_column += 1;
            }
            _ => {
                result.push(current_char);
                current_column += 1;
            }
        }
    }
    result
}

// Returns the length of the longest variant key name.
fn find_longest_variant(text: &str) -> usize {
    let mut current_size: usize = 0;
    let mut longest_var: usize = 0;

    let mut iter = text.chars().peekable();

    while let Some(current_char) = iter.next() {
        match current_char {
            '{' | ',' | '\n' => {
                current_size = 0;
            }
            ':' => {
                if current_size > longest_var {
                    longest_var = current_size;
                }
            }
            _ => {
                current_size += 1;
            }
        }
    }
    longest_var
}

#[test]
fn test_find_longest_variant() {
    let raw = r#"struct MyStruct {
        foo: u32,
        foooooo: u32, 
        bar: u64,
    }"#;
    assert_eq!(find_longest_variant(raw), 15);
    let raw = r#"enum myenum {     foo: u32,
        AH: u32, 
        thisisatest: u64
    }"#;
    assert_eq!(find_longest_variant(raw), 19);
    let raw = r#"enum myenum {    // test comment
        b: u32, 
        a: u64
    }"#;
    assert_eq!(find_longest_variant(raw), 9);
    let raw = r#"enum myenum { // comment
        b /*hi comment test */ : u32, 
        a: u64
    }"#;
    assert_eq!(find_longest_variant(raw), 31);
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

/// Given text right after a `:` in an enum or struct, get the type as a string.
/// If this function is given data that either starts with a `:` or is the field/variant _name_, it will not work.
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
                    // type names cannot have spaces
                    ' ' => {
                        iter.next();
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
