use sway_core::{compile_to_ast, namespace, Engines};
use sway_error::handler::Handler;
use sway_features::ExperimentalFeatures;
use sway_types::{Ident, ProgramId};

#[test]
fn parenthesized_struct_match_type_checks() {
    let engines = Engines::default();
    let handler = Handler::default();
    let package = namespace::Package::new(
        Ident::new_no_span("match_struct".to_string()),
        None,
        ProgramId::new(0),
        false,
    );
    let result = compile_to_ast(
        &handler,
        &engines,
        "library;
        struct S { x: u64 }
        pub fn f() -> u64 {
            let _ = match (S { x: 0 }) { _ => 0, };
            match (S { x: 42 }) { S { x } => x, }
        }"
        .into(),
        package,
        None,
        "match_struct",
        None,
        ExperimentalFeatures::default(),
    );
    let (errors, _, _) = handler.consume();
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(result.unwrap().typed.is_ok());
}
