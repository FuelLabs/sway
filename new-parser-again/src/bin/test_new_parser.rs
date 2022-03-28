use {
    std::{
        path::Path,
        sync::Arc,
    },
    new_parser_again::Parser,
};

static BAD: &[&str] = &[
    "recursive_calls",
    "asm_missing_return",
    "asm_should_not_have_return",
    "missing_fn_arguments",
    "excess_fn_arguments",
    // the feature for the below test, detecting inf deps, was reverted
    // when that is re-implemented we should reenable this test
    //"infinite_dependencies",
    "top_level_vars",
    "dependency_parsing_error",
    "disallowed_gm",
    "bad_generic_annotation",
    "bad_generic_var_annotation",
    "unify_identical_unknowns",
    "array_oob",
    "array_bad_index",
    "name_shadowing",
    "match_expressions_wrong_struct",
    "match_expressions_enums",
    "pure_calls_impure",
    "nested_impure",
    "predicate_calls_impure",
    "script_calls_impure",
    "contract_pure_calls_impure",
    "literal_too_large_for_type",
    "star_import_alias",
    "item_used_without_import",
    "shadow_import",
    "missing_supertrait_impl",
    "missing_func_from_supertrait_impl",
    "supertrait_does_not_exist",
];

fn main() {
    let repo_dir = {
        let mut dir = std::env::current_dir().unwrap();
        while dir.file_name().unwrap() != "sway" {
            dir.pop();
        }
        dir
    };
    let test_dir = {
        let mut dir = repo_dir.clone();
        dir.push("test");
        dir.push("src");
        dir.push("e2e_vm_tests");
        dir.push("test_programs");
        dir
    };
    if !parse_all_in_dir(&test_dir) {
        return;
    }
    let libstd_dir = {
        let mut dir = repo_dir.clone();
        dir.pop();
        dir.push("sway-lib-std");
        dir
    };
    if !parse_all_in_dir(&libstd_dir) {
        return;
    }
}

fn parse_all_in_dir(dir: &Path) -> bool {
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
            BAD
            .iter()
            .any(|bad| path.to_str().unwrap().contains(bad))
        } {
            continue;
        }

        let src = {
            let src = std::fs::read(path).unwrap();
            let src = String::from_utf8(src).unwrap();
            Arc::from(src)
        };
        println!("lexing: {}", path.display());
        let path = Arc::new(path.to_owned());
        let lex_res = new_parser_again::lex(&src, 0, src.len(), Some(path.clone()));
        let token_stream = match lex_res {
            Ok(token_stream) => token_stream,
            Err(error) => {
                println!("lex error: {:?}", error);
                return false;
            },
        };
        println!("parsing: {}", path.display());
        let mut errors = Vec::new();
        let parser = Parser::new(&token_stream, &mut errors);
        let program_res = parser.parse_to_end::<new_parser_again::Program>();
        let _program = match program_res {
            Ok(program) => program,
            Err(_error) => {
                println!("parse errors:");
                for error in errors {
                    println!("{}", error);
                }
                return false;
            },
        };
        println!("ok!");
    }
    true
}
