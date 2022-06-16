mod compile;
mod const_eval;
mod convert;
mod function;
mod lexical_map;
mod purity;
mod types;

use crate::{
    error::CompileError,
    semantic_analysis::{TypedProgram, TypedProgramKind},
};

use sway_ir::Context;
use sway_types::span::Span;

pub(crate) use purity::PurityChecker;

pub(crate) fn compile_program(program: TypedProgram) -> Result<Context, CompileError> {
    let TypedProgram { kind, root } = program;

    let mut ctx = Context::default();
    match kind {
        TypedProgramKind::Script {
            main_function,
            declarations,
        }
        | TypedProgramKind::Predicate {
            main_function,
            declarations,
            // predicates and scripts have the same codegen, their only difference is static
            // type-check time checks.
        } => compile::compile_script(&mut ctx, main_function, &root.namespace, declarations),
        TypedProgramKind::Contract {
            abi_entries,
            declarations,
        } => compile::compile_contract(&mut ctx, abi_entries, &root.namespace, declarations),
        TypedProgramKind::Library { .. } => unimplemented!("compile library to ir"),
    }?;
    ctx.verify()
        .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))
}

#[cfg(test)]
mod tests {
    use crate::semantic_analysis::{namespace, TypedProgram};
    use std::path::PathBuf;

    #[test]
    fn sway_to_ir_tests() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dir: PathBuf = format!("{}/tests/sway_to_ir", manifest_dir).into();
        for entry in std::fs::read_dir(dir).unwrap() {
            // We're only interested in the `.sw` files here.
            let path = entry.unwrap().path();
            match path.extension().unwrap().to_str() {
                Some("sw") => {
                    //
                    // Run the tests!
                    //
                    tracing::info!("---- Sway To IR: {:?} ----", path);
                    test_sway_to_ir(path);
                }
                Some("ir") | Some("disabled") => (),
                _ => panic!(
                    "File with invalid extension in tests dir: {:?}",
                    path.file_name().unwrap_or(path.as_os_str())
                ),
            }
        }
    }

    fn test_sway_to_ir(sw_path: PathBuf) {
        let input_bytes = std::fs::read(&sw_path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        let mut ir_path = sw_path.clone();
        ir_path.set_extension("ir");

        let expected_bytes = std::fs::read(&ir_path).unwrap();
        let expected = String::from_utf8_lossy(&expected_bytes);

        let typed_program = parse_to_typed_program(sw_path.clone(), &input);
        let ir = super::compile_program(typed_program).unwrap();
        let output = sway_ir::printer::to_string(&ir);

        // Use a tricky regex to replace the local path in the metadata with something generic.  It
        // should convert, e.g.,
        //     `!0 = filepath "/usr/home/me/sway/sway-core/tests/sway_to_ir/foo.sw"`
        //  to `!0 = filepath "/path/to/foo.sw"`
        let path_converter = regex::Regex::new(r#"(!\d = filepath ")(?:[^/]*/)*(.+)"#).unwrap();
        let output = path_converter.replace_all(output.as_str(), "$1/path/to/$2");

        if output != expected {
            println!("{}", prettydiff::diff_lines(&expected, &output));
            panic!("{} failed.", sw_path.display());
        }
    }

    #[test]
    fn ir_printer_parser_tests() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dir: PathBuf = format!("{}/tests/sway_to_ir", manifest_dir).into();
        for entry in std::fs::read_dir(dir).unwrap() {
            // We're only interested in the `.ir` files here.
            let path = entry.unwrap().path();
            match path.extension().unwrap().to_str() {
                Some("ir") => {
                    //
                    // Run the tests!
                    //
                    tracing::info!("---- IR Print and Parse Test: {:?} ----", path);
                    test_printer_parser(path);
                }
                Some("sw") | Some("disabled") => (),
                _ => panic!(
                    "File with invalid extension in tests dir: {:?}",
                    path.file_name().unwrap_or(path.as_os_str())
                ),
            }
        }
    }

    fn test_printer_parser(path: PathBuf) {
        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        // Use another tricky regex to inject the proper metadata filepath back, so we can create
        // spans in the parser.  NOTE, if/when we refactor spans to not have the source string and
        // just the path these tests should pass without needing this conversion.
        let mut true_path = path.clone();
        true_path.set_extension("sw");
        let path_converter = regex::Regex::new(r#"(!\d = filepath )(?:.+)"#).unwrap();
        let input = path_converter.replace_all(&input, format!("$1\"{}\"", true_path.display()));

        let parsed_ctx = match sway_ir::parser::parse(&input) {
            Ok(p) => p,
            Err(e) => {
                println!("{}: {}", path.display(), e);
                panic!();
            }
        };
        let printed = sway_ir::printer::to_string(&parsed_ctx);
        if printed != input {
            println!("{}", prettydiff::diff_lines(&input, &printed));
            panic!("{} failed.", path.display());
        }
    }

    fn parse_to_typed_program(path: PathBuf, input: &str) -> TypedProgram {
        let root_module = std::sync::Arc::new(path);
        let canonical_root_module = std::sync::Arc::new(root_module.canonicalize().unwrap());

        let build_config = crate::build_config::BuildConfig {
            canonical_root_module,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
        };
        let mut warnings = vec![];
        let mut errors = vec![];
        let src = std::sync::Arc::from(input);
        let parsed_program =
            crate::parse(src, Some(&build_config)).unwrap(&mut warnings, &mut errors);

        let initial_namespace = namespace::Module::default();
        let typed_program = TypedProgram::type_check(parsed_program, initial_namespace)
            .unwrap(&mut warnings, &mut errors);

        crate::perform_control_flow_analysis(&typed_program).unwrap(&mut warnings, &mut errors);

        typed_program
    }
}
