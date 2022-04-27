use super::{FunctionDeclaration, TypeParameter};
use crate::{
    build_config::BuildConfig, error::*, parse_tree::CallPath, parser::Rule, type_engine::TypeInfo,
};

use sway_types::span::Span;

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImplTrait {
    pub trait_name: CallPath,
    pub(crate) type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) type_arguments: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelf {
    pub type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

impl ImplTrait {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let path = config.map(|c| c.path());
        let block_span = Span::from_pest(pair.as_span(), path.clone());

        let mut iter = pair.into_inner().peekable();

        let impl_keyword = iter.next().unwrap();
        assert_eq!(impl_keyword.as_str(), "impl");

        // see if there are any generic type params
        let type_params_pair = match iter.peek() {
            Some(pair) if pair.as_rule() == Rule::type_params => Some(iter.next().unwrap()),
            _ => None,
        };

        // get the trait name
        let trait_name = iter.next().unwrap();
        let trait_name = check!(
            CallPath::parse_from_pair(trait_name, config),
            return err(warnings, errors),
            warnings,
            errors
        );

        // construct the type that we are implementing for
        let type_name = iter.next().unwrap();
        let type_implementing_for_span = Span::from_pest(type_name.as_span(), path);
        let type_implementing_for = check!(
            TypeInfo::parse_from_pair(type_name, config),
            return err(warnings, errors),
            warnings,
            errors
        );

        // see if there are any trait bounds
        let where_clause_pair = match iter.peek() {
            Some(pair) if pair.as_rule() == Rule::trait_bounds => Some(iter.next().unwrap()),
            _ => None,
        };

        // construct the type arguments
        let type_arguments = check!(
            TypeParameter::parse_from_type_params_and_where_clause(
                type_params_pair,
                where_clause_pair,
                config
            ),
            vec!(),
            warnings,
            errors
        );

        // collect the methods in the impl
        let mut fn_decls_buf = vec![];
        for pair in iter {
            fn_decls_buf.push(check!(
                FunctionDeclaration::parse_from_pair(pair, config),
                continue,
                warnings,
                errors
            ));
        }

        ok(
            ImplTrait {
                trait_name,
                type_arguments,
                type_implementing_for,
                type_implementing_for_span,
                functions: fn_decls_buf,
                block_span,
            },
            warnings,
            errors,
        )
    }
}

impl ImplSelf {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let path = config.map(|c| c.path());
        let block_span = Span::from_pest(pair.as_span(), path.clone());

        let mut iter = pair.into_inner().peekable();

        let impl_keyword = iter.next().unwrap();
        assert_eq!(impl_keyword.as_str(), "impl");

        // see if there are any generic type params
        let type_params_pair = match iter.peek() {
            Some(pair) if pair.as_rule() == Rule::type_params => Some(iter.next().unwrap()),
            _ => None,
        };

        // construct the type that we are implementing for
        let type_name = iter.next().unwrap();
        let type_implementing_for_span = Span::from_pest(type_name.as_span(), path);
        let type_implementing_for = check!(
            TypeInfo::parse_from_pair(type_name, config),
            return err(warnings, errors),
            warnings,
            errors
        );

        // see if there are any trait bounds
        let where_clause_pair = match iter.peek() {
            Some(pair) if pair.as_rule() == Rule::trait_bounds => Some(iter.next().unwrap()),
            _ => None,
        };

        // construct the type arguments
        let type_arguments = check!(
            TypeParameter::parse_from_type_params_and_where_clause(
                type_params_pair,
                where_clause_pair,
                config
            ),
            vec!(),
            warnings,
            errors
        );

        // collect the methods in the impl
        let mut fn_decls_buf = vec![];
        for pair in iter {
            fn_decls_buf.push(check!(
                FunctionDeclaration::parse_from_pair(pair, config),
                continue,
                warnings,
                errors
            ));
        }

        ok(
            ImplSelf {
                type_implementing_for,
                type_implementing_for_span,
                type_parameters: type_arguments,
                functions: fn_decls_buf,
                block_span,
            },
            warnings,
            errors,
        )
    }
}
