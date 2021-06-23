use super::code_builder::CodeBuilder;

/// returns number of lines and formatted text
pub fn get_formatted_data(file: &str, tab_size: u32) -> (usize, String) {
    let mut code_builder = CodeBuilder::new(tab_size);
    let lines: Vec<&str> = file.split("\n").collect();

    // todo: handle lengthy lines of code
    for line in lines {
        code_builder.format_and_add(line);
    }

    code_builder.get_final_edits()
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
        let (_, result) = get_formatted_data(correct_sway_code, 4);
        assert_eq!(correct_sway_code, result);

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

        let (_, result) = get_formatted_data(sway_code, 4);
        assert_eq!(correct_sway_code, result);
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

        let (_, result) = get_formatted_data(correct_sway_code, 4);
        assert_eq!(correct_sway_code, result);

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

        let (_, result) = get_formatted_data(sway_code, 4);
        assert_eq!(correct_sway_code, result);
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

        let (_, result) = get_formatted_data(correct_sway_code, 4);
        assert_eq!(correct_sway_code, result);

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

        let (_, result) = get_formatted_data(sway_code, 4);
        assert_eq!(correct_sway_code, result);
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
    age: u32 // another comment
} // comment as well
"#;

        let (_, result) = get_formatted_data(correct_sway_code, 4);
        assert_eq!(correct_sway_code, result);

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

        let (_, result) = get_formatted_data(sway_code, 4);
        assert_eq!(correct_sway_code, result);
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

struct Vehicle {
    age: u32,
    name: string, // some comment middle of nowhere
}

struct Environment {
    age: u32,
    name: string
} // lost my train of thought

struct Person { // first comment
    age: u32, // second comment
    name: string, // third comment
} // fourth comment
"#;

        let (_, result) = get_formatted_data(correct_sway_code, 4);
        assert_eq!(correct_sway_code, result);

        let sway_code = r#"script;

fn main() {

}

struct Structure {
    age: u32,
    name: string,
}

struct Vehicle 
          { age:       u32,          name: string , // some comment middle of nowhere
}

struct Environment{age:u32,name:string} // lost my train of thought

struct Person {// first comment
    age: u32,// second comment
    name: string,          // third comment
} // fourth comment
"#;

        let (_, result) = get_formatted_data(sway_code, 4);
        assert_eq!(correct_sway_code, result);
    }
}
