use crate::code_builder_helpers::*;
use crate::constants::{ALREADY_FORMATTED_LINE_PATTERN, NEW_LINE_PATTERN, TAB_SIZE};
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
    struct /* i am about to declare a struct */ DummyStruct { // hi
        // properly handling comments?
sumn /* hi i am  a comment */ : value /* another comment */,
    sumnelse: u32, // so many comments
nocomments: u64,
    /* hi */    } // oops a comment
    ";
    let formatted = r"
    struct /* i am about to declare a struct */ DummyStruct {
        // properly handling comments?
        sumn /* hi i am  a comment */ : value /* another comment */,
        sumnelse                      : u32, // so many comments
        nocomments                    : u64,
        /* hi */    
    } // oops a comment
    ";
    let post_formatting = format_align_data_types(unformatted_struct);
    println!("{}", post_formatting);
    assert_eq!(post_formatting, formatted);
}

/// Formats Sway data types and aligns fields for Enums and Structs
pub fn format_align_data_types(text: &str) -> String {
    let longest_var = find_longest_variant(text);
    let mut current_column = 0;
    let mut iter = text.chars().enumerate().peekable();
    let mut result = String::new();
    let newline_and_tab = format!("\n{}", {
        let buf = vec![" "; TAB_SIZE];
        buf.join("")
    });
    clean_all_whitespace_enumerated(&mut iter);

    while let Some((_, current_char))  = iter.next() {
        match dbg!(current_char) {
            '}' => {
                clean_all_whitespace_enumerated(&mut iter);
                if current_column != 0 {
                    result.push('\n');
                }
                result.push(current_char);
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
                result.push_str(&newline_and_tab);
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
                    },
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
