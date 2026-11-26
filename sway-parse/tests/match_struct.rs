use sway_ast::Expr;
use sway_error::{diagnostic::ToDiagnostic, error::CompileError, handler::Handler};
use sway_features::ExperimentalFeatures;
use sway_parse::{lex, parse_file, Parser};
use sway_types::SourceEngine;

#[test]
fn match_struct_instantiation_requires_parentheses() {
    for initializer in ["S { x: 0 }", "S { x }", "module::S { x: S { x: 0 }, }"] {
        let input = format!("match {initializer} {{ _ => 0, }}");
        let handler = Handler::default();
        let tokens = lex(&handler, input.as_str().into(), 0, input.len(), None).unwrap();
        let result =
            Parser::new(&handler, &tokens, ExperimentalFeatures::default()).parse::<Expr>();
        assert!(result.is_err());
        let (errors, warnings, _) = handler.consume();
        assert!(warnings.is_empty());
        assert_eq!(errors.len(), 1, "{errors:?}");
        let CompileError::Parse { error } = &errors[0] else {
            panic!("Expected a parser error: {errors:?}");
        };
        assert_eq!(error.span.as_str(), initializer);
        assert_eq!(
            error.kind.to_string(),
            "Struct instantiations in match expressions must be wrapped in parentheses."
        );
        let diagnostic = errors[0].to_diagnostic(&SourceEngine::default());
        assert_eq!(
            diagnostic.help,
            vec![format!(
                "Wrap the struct instantiation in parentheses: `match ({initializer}) {{ ... }}`."
            )]
        );
    }
}

#[test]
fn valid_match_forms_are_preserved() {
    for statement in [
        "let _ = match (S { x: 0 }) { _ => 0, };",
        "let _ = match s { S { x } => x, };",
        "let _ = match s { x => x, };",
        "let _ = match f(S { x: 0 }) { _ => 0, };",
        "match s {} {}",
        "match s { _ => 0, } {}",
    ] {
        let input = format!("script; fn main() {{ {statement} }}");
        let handler = Handler::default();
        let result = parse_file(
            &handler,
            input.as_str().into(),
            None,
            ExperimentalFeatures::default(),
        );
        let (errors, warnings, _) = handler.consume();
        assert!(result.is_ok(), "{statement}: {errors:?}");
        assert!(errors.is_empty(), "{statement}: {errors:?}");
        assert!(warnings.is_empty());
    }
}

#[test]
fn unrelated_match_errors_are_preserved() {
    for input in [
        "match s { x: 0 }",
        "match s { S { x } } {}",
        "match s { x + 1 } {}",
    ] {
        let handler = Handler::default();
        let tokens = lex(&handler, input.into(), 0, input.len(), None).unwrap();
        assert!(
            Parser::new(&handler, &tokens, ExperimentalFeatures::default())
                .parse::<Expr>()
                .is_err()
        );
        let (errors, _, _) = handler.consume();
        assert_eq!(errors.len(), 1, "{input}: {errors:?}");
        let CompileError::Parse { error } = &errors[0] else {
            panic!("Expected a parser error: {errors:?}");
        };
        assert_eq!(error.kind.to_string(), "Expected `=>`.", "{input}");
    }
}

#[test]
fn nested_malformed_matches_do_not_repeat_arm_parsing() {
    // Recursive expression parsing needs more than the default test thread stack.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let mut input = "match s { x + 1 } {}".to_string();
            for _ in 0..20 {
                input = format!("match s {{ _ => {input}, }} {{}}");
            }
            let handler = Handler::default();
            let tokens = lex(&handler, input.as_str().into(), 0, input.len(), None).unwrap();
            assert!(
                Parser::new(&handler, &tokens, ExperimentalFeatures::default())
                    .parse::<Expr>()
                    .is_err()
            );
            let (errors, _, _) = handler.consume();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].to_string(), "Expected `=>`.");
        })
        .unwrap()
        .join()
        .unwrap();
}
