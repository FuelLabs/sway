use {
    std::sync::Arc,
    new_parser_again::Parser,
};

fn main() {
    let dir = {
        let mut dir = std::env::current_dir().unwrap();
        while dir.file_name().unwrap() != "sway" {
            dir.pop();
        }
        dir.push("test");
        dir.push("src");
        dir.push("e2e_vm_tests");
        dir.push("test_programs");
        dir
    };
    let mut good = 0;
    let mut bad = 0;
    for entry_res in walkdir::WalkDir::new(&dir).sort_by_file_name() {
        let entry = entry_res.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        match path.extension() {
            Some(extension) if extension == "sw" => (),
            _ => continue,
        }
        if {
            path.to_str().unwrap().contains("parsing_error") ||
            path.to_str().unwrap().contains("top_level_vars") 
        } {
            continue;
        }

        let src = {
            let src = std::fs::read(path).unwrap();
            let src = String::from_utf8(src).unwrap();
            Arc::from(src)
        };
        println!("lexing: {}", path.display());
        let lex_res = new_parser_again::lex(&src);
        let token_stream = match lex_res {
            Ok(token_stream) => token_stream,
            Err(error) => {
                println!("lex error: {:?}", error);
                bad += 1;
                //continue;
                break;
            },
        };
        println!("parsing: {}", path.display());
        let parser = Parser::new(&token_stream);
        let program_res = parser.parse_to_end::<new_parser_again::Program>();
        let _program = match program_res {
            Ok(program) => program,
            Err(_error) => {
                bad += 1;
                /*
                continue;
                */
                break;
            },
        };
        good += 1;
        println!("ok!");
    }
    println!("good == {}", good);
    println!("bad == {}", bad);
}
