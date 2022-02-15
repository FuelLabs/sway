use super::code_builder::CodeBuilder;
use crate::traversal::{traverse_for_changes, Change};
use ropey::Rope;
use std::sync::Arc;

/// Returns number of lines and formatted text.
/// Formatting is done as a 2-step process.
/// Firstly, certain Sway types (like Enums and Structs) are formatted separately in isolation,
/// depending on their type.
/// Secondly, after that, the whole file is formatted/cleaned and checked
/// for smaller things like extra newlines, indentation and similar.
pub fn get_formatted_data(
    file: Arc<str>,
    formatting_options: FormattingOptions,
) -> Result<(usize, String), Vec<String>> {
    let parsed_res = sway_core::parse(file.clone(), None);

    match parsed_res.value {
        Some(parse_tree) => {
            // 1 Step: get all individual changes/updates of a Sway file
            let changes = traverse_for_changes(&parse_tree);
            let mut rope_file = Rope::from_str(&file);

            let mut offset: i32 = 0;
            for change in changes {
                // for each update, calculate their newly position
                // and add it in the existing file
                let (new_offset, start, end) = calculate_offset(offset, &change);
                offset = new_offset;

                rope_file.remove(start..end);
                rope_file.insert(start, &change.text);
            }

            // 2 Step: CodeBuilder goes through each line of a Sway file and cleans it up
            let mut code_builder = CodeBuilder::new(formatting_options.tab_size);

            let file = rope_file.to_string();
            let lines: Vec<&str> = file.split('\n').collect();

            // todo: handle lengthy lines of code
            for line in lines {
                code_builder.format_and_add(line);
            }

            Ok(code_builder.get_final_edits())
        }
        None => Err(parsed_res
            .errors
            .iter()
            .map(|e| {
                format!(
                    "{:?} at line: {}",
                    e.to_friendly_error_string(),
                    e.line_col().0.line,
                )
            })
            .collect()),
    }
}

fn calculate_offset(current_offset: i32, change: &Change) -> (i32, usize, usize) {
    let start = change.start as i32 + current_offset;
    let end = change.end as i32 + current_offset;
    let offset = current_offset + (start + change.text.len() as i32) - end;

    (offset, start as usize, end as usize)
}

#[derive(Debug, Clone, Copy)]
pub struct FormattingOptions {
    pub align_fields: bool,
    pub tab_size: u32,
}

impl FormattingOptions {
    pub fn default() -> Self {
        Self {
            align_fields: true,
            tab_size: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FormattingOptions;

    use super::get_formatted_data;
    const OPTIONS: FormattingOptions = FormattingOptions {
        align_fields: false,
        tab_size: 4,
    };

    #[test]
    fn test_indentation() {
        let correct_sway_code = r#"script;

fn main() {
    // this is a comment
    let o = 123;

    let p = {
        /* this is some
            multi line stuff t
        
        */
        123;
    };

    add(1, 2);
}

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
"#;
        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

fn main() {
    // this is a comment
    let o = 123;

                let 
p   
    
    
            =
    
    
        {
        /* this is some
            multi line stuff t
        
        */
        123       
        
        
                        ;
    
    
    };

    add(        1,    
    
                                                        2 
    
    
            )     ;
}

pub
fn 
add
    (
    a:u32   , 
            b: u32)             ->u32{
    a +b}

"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_multiline_string() {
        let correct_sway_code = r#"script;

fn main() {
    let multiline_string = "       sadsa
    sadsad
        sadasd sadsdsa
    sadasd
        sadasd sadasd
    ";
}
"#;

        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

fn main(){
    let multiline_string="       sadsa
    sadsad
        sadasd sadsdsa
    sadasd
        sadasd sadasd
    "          
               ;
}
"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_whitespace_handling() {
        let correct_sway_code = r#"script;

fn main() {
    let word = "word";
    let num = 12;

    let multi = {
        let k = 12;
        k
    };
}
"#;

        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

fn main() {
    let word="word";
    let num=               12           ;

    let multi = {
        let k         = 12;
                    k
    }
    
    
                ;
}
"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_comments() {
        let correct_sway_code = r#"script;

fn main() {
    // this is a comment
    let o = 123; // this is an inline comment
    /*
        asdasd
    asdasdsad asdasdasd */

    /* multiline closed on the same line */
    let p = {
        /* this is some
            multi line stuff t
        
         */
        123;
    }; // comment here as well
} // comment here too

// example struct with comments
struct Example { // first comment
    prop: bool, // second comment
    age: u32, // another comment
} // comment as well
"#;

        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

fn main() {
    // this is a comment
    let o = 123;            // this is an inline comment
     /*
        asdasd
    asdasdsad asdasdasd */

         /* multiline closed on the same line */
    let p = {
        /* this is some
            multi line stuff t
        
         */
        123;
    };     // comment here as well
} // comment here too

 // example struct with comments
struct Example {    // first comment
    prop: bool,// second comment
    age: u32// another comment
}   // comment as well
"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_data_types() {
        let correct_sway_code = r#"script;

fn main() {
    let rgb: Rgb = Rgb {
        red: 255,
        blue: 0,
        green: 0,
    };

    if (true) {
        let rgb: Rgb = Rgb {
            red: 255,
            blue: 0,
            green: 0,
        };
    }
}

struct Rgb {
    red: u64,
    green: u64,
    blue: u64,
}

struct Structure {
    age: u32,

    name: string,
}

struct Structure {
    age: u32, /* completely meaningless multiline comment
        not sure why would anyone write this but let's deal with it as well!
    */
    name: string,
}

struct Structure {
    age: u32,
    name: string, // super comment
}

struct Structure {
    age: u32,
    name: string, // super comment
}

struct Vehicle {
    age: u32,
    name: string, // some comment middle of nowhere
}

struct Environment {
    age: u32,
    name: string,
} // lost my train of thought

struct Person { // first comment
    age: u32, // second comment
    name: string, // third comment
} // fourth comment

pub fn get_age() -> u32 {
    99
}

pub fn read_example() -> Example {
    Example {
        age: get_age(),
        name: "Example face",
    }
}

struct Example {
    age: u32,
    name: string,
}

struct C {
    /// a docstring
    a: A,
    /// b docstring
    b: byte,
}

struct A {
    a: u64,
    b: u64,
}

fn get_gas() -> A {
    A {
        a: asm() {
            ggas
        },
        b: asm() {
            cgas
        },
    }
}
"#;

        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

fn main() {
    let rgb:Rgb=Rgb {
        red: 255,
        blue: 0,
        green: 0,
    };

    if(true){
        let rgb: Rgb = Rgb {
            red:255,      blue: 0,
                green: 0,
        };
    }
}

struct Rgb {
    red: u64,
    green: u64,
    blue: u64,
}

struct Structure {

    age: u32,

    name: string,

}

struct Structure {
    age: u32, /* completely meaningless multiline comment
        not sure why would anyone write this but let's deal with it as well!
    */
    name: string
}

struct Structure {
    age: u32,
    name: string// super comment
}

struct Structure {
    age: u32,
    name: string, // super comment
}

struct Vehicle 
          { age:       u32,          name: string , // some comment middle of nowhere
}

struct Environment{age:u32,name:string} // lost my train of thought

struct Person {// first comment
    age: u32,// second comment
    name: string,          // third comment
} // fourth comment

pub fn get_age() -> u32 {
     99
}

pub fn read_example() -> Example {
    Example {
        age: get_age()     ,name: "Example face"
    }
}

struct Example {age: u32,    name: string}

struct C {
/// a docstring
a: A,
/// b docstring
b: byte,
}

struct A {
a: u64,
b: u64,
}

fn get_gas() -> A {
A {
a: asm() {
ggas
},
b: asm() {
cgas
}
}
}
"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_enums() {
        let correct_sway_code = r#"script;

pub fn main() {
    let k = Story::Pain;
}

enum Story {
    Pain: (),
    Gain: (),
}

enum StoryA {
    Pain: (),
    Gain: (),
}

pub fn tell_a_story() -> Story {
    Story::Gain
}
"#;

        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

        pub fn main() {
            let k = 
                    Story          :: Pain;
        
        
        
        }
        
        
        enum Story {
           Pain:(),
            Gain:()
        }
        
        enum StoryA {Pain:(),Gain:()}
        
        pub fn tell_a_story() ->Story {
                Story   :: Gain
        }
"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_comparison_operators() {
        let correct_sway_code = r#"script;

fn main() {
    if 1 >= 0 {
    } else if 4 <= 0 {
    } else if 5 == 0 {
    } else if 4 != 4 {
    } else {
    }
}

fn one_liner() -> bool {
    if 1 >= 0 {
        true
    } else if 1 <= 0 {
        true
    } else {
        true
    }
}
"#;

        let result = get_formatted_data(correct_sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

        fn main() {
            if 1 >= 0 {
        
            }       else if 4 <= 0 
            
            {
        
            } 
        else if 5 == 0 { } else if 4 != 4 
        
        {
        
            } 
                    else {
        
            }    
        }        

        fn one_liner() -> bool {
            if 1 >= 0 { true } else if 1 <= 0 { true } else { true }
        }
"#;

        let result = get_formatted_data(sway_code.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    // Test that the use statements with multiple imports are properly formatted
    fn test_use_statement() {
        let test_sway = r#"script;
use std::chain::{panic,log_u8};
use std::chain::assert;
use std::hash::{HashMethod,    hash_value,               hash_pair    };
use a::b::{c,d::{f,e}};
use a::b::{c,d::{f,self}};

fn main() {
}
"#;
        let expected_sway = r#"script;
use std::chain::{log_u8, panic};
use std::chain::assert;
use std::hash::{HashMethod, hash_pair, hash_value};
use a::b::{c, d::{e, f}};
use a::b::{c, d::{self, f}};

fn main() {
}
"#;
        let result = get_formatted_data(test_sway.into(), OPTIONS);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(formatted_code, expected_sway);
    }
}
