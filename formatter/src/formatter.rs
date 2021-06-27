use super::code_builder::CodeBuilder;
use crate::traversal::{traverse_for_changes, Change};
use ropey::Rope;

/// returns number of lines and formatted text
pub fn get_formatted_data(file: &str, tab_size: u32) -> Result<(usize, String), &str> {
    match core_lang::parse(&file) {
        core_lang::CompileResult::Ok {
            value: parse_tree,
            warnings: _,
            errors: _,
        } => {
            let changes = traverse_for_changes(&parse_tree);
            let mut rope_file = Rope::from_str(file);

            let mut offset: i32 = 0;
            for change in changes {
                let (new_offset, start, end) = calculate_offset(offset, &change);
                offset = new_offset;

                rope_file.remove(start..end);
                rope_file.insert(start, &change.text);
            }

            let mut code_builder = CodeBuilder::new(tab_size);

            let file = rope_file.to_string();
            let lines: Vec<&str> = file.split("\n").collect();

            // todo: handle lengthy lines of code
            for line in lines {
                code_builder.format_and_add(line);
            }

            Ok(code_builder.get_final_edits())
        }
        _ => Err("Failed to parse the file"),
    }
}

fn calculate_offset(current_offset: i32, change: &Change) -> (i32, usize, usize) {
    let start = change.start as i32 + current_offset;
    let end = change.end as i32 + current_offset;
    let offset = current_offset + (start + change.text.len() as i32) - end;

    (offset, start as usize, end as usize)
}

#[cfg(test)]
mod tests {
    use super::get_formatted_data;

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
        let result = get_formatted_data(correct_sway_code, 4);
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

        let result = get_formatted_data(sway_code, 4);
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

        let result = get_formatted_data(correct_sway_code, 4);
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

        let result = get_formatted_data(sway_code, 4);
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

        let result = get_formatted_data(correct_sway_code, 4);
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

        let result = get_formatted_data(sway_code, 4);
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

        let result = get_formatted_data(correct_sway_code, 4);
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

        let result = get_formatted_data(sway_code, 4);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }

    #[test]
    fn test_custom_types() {
        let correct_sway_code = r#"script;

fn main() {

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
"#;

        let result = get_formatted_data(correct_sway_code, 4);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);

        let sway_code = r#"script;

fn main() {

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
"#;

        let result = get_formatted_data(sway_code, 4);
        assert!(result.is_ok());
        let (_, formatted_code) = result.unwrap();
        assert_eq!(correct_sway_code, formatted_code);
    }
}
