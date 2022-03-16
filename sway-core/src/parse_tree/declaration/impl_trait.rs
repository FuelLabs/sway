use super::{FunctionDeclaration, TypeParameter};
use crate::{
    build_config::BuildConfig, error::*, parse_tree::CallPath, parser::Rule, type_engine::TypeInfo,
};

use sway_types::span::Span;

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImplTrait {
    pub(crate) trait_name: CallPath,
    pub(crate) type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) type_arguments: Vec<TypeParameter>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
    pub(crate) type_arguments_span: Span,
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelf {
    pub(crate) type_implementing_for: TypeInfo,
    pub(crate) type_implementing_for_span: Span,
    pub(crate) generic_type_arguments: Vec<TypeParameter>,
    #[allow(dead_code)]
    generic_type_arguments_span: Option<Span>,
    #[allow(dead_code)]
    pub(crate) specific_type_arguments: Vec<TypeInfo>,
    #[allow(dead_code)]
    specific_type_arguments_span: Option<Span>,
    pub functions: Vec<FunctionDeclaration>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span,
}

impl ImplTrait {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let block_span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut iter = pair.into_inner();
        let impl_keyword = iter.next().unwrap();
        assert_eq!(impl_keyword.as_str(), "impl");
        let trait_name = iter.next().unwrap();
        let trait_name = check!(
            CallPath::parse_from_pair(trait_name, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut iter = iter.peekable();
        let type_arguments_pair = if iter.peek().unwrap().as_rule() == Rule::type_params {
            iter.next()
        } else {
            None
        };

        let type_implementing_for_pair = iter.next().expect("guaranteed by grammar");
        let type_implementing_for_span = Span {
            span: type_implementing_for_pair.as_span(),
            path: path.clone(),
        };
        let type_implementing_for = check!(
            TypeInfo::parse_from_pair(type_implementing_for_pair, config),
            return err(warnings, errors),
            warnings,
            errors
        );

        let where_clause_pair = match iter.peek() {
            Some(r) => match r.as_rule() {
                Rule::trait_bounds => iter.next(),
                _ => None,
            },
            None => None,
        };

        let type_arguments_span = match type_arguments_pair {
            Some(ref x) => Span {
                span: x.as_span(),
                path,
            },
            None => trait_name.span(),
        };
        let type_arguments = TypeParameter::parse_from_type_params_and_where_clause(
            type_arguments_pair,
            where_clause_pair,
            config,
        )
        .unwrap_or_else(&mut warnings, &mut errors, Vec::new);

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
                type_arguments_span,
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
        let block_span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };

        let mut iter = pair.into_inner().peekable();

        let impl_keyword = iter.next().unwrap();
        assert_eq!(impl_keyword.as_str(), "impl");

        // get generic type arguments
        let generic_type_arguments = if iter.peek().unwrap().as_rule() == Rule::type_params {
            iter.next()
        } else {
            None
        };

        // get the type we are implementing for
        let type_name = iter.next().unwrap();
        let type_implementing_for_span = Span {
            span: type_name.as_span(),
            path: path.clone(),
        };
        let mut type_name_iter = type_name.into_inner().peekable();
        let type_implementing_for = check!(
            TypeInfo::parse_from_pair(type_name_iter.next().unwrap(), config),
            return err(warnings, errors),
            warnings,
            errors
        );

        // get the actual type arguments
        let specific_type_arguments = type_name_iter.next();

        // get the type argument spans
        let generic_type_arguments_span = generic_type_arguments.clone().map(|ref x| Span {
            span: x.as_span(),
            path: path.clone(),
        });
        let specific_type_arguments_span = specific_type_arguments.clone().map(|ref x| Span {
            span: x.as_span(),
            path: path.clone(),
        });

        // see if there are any trait bounds
        let trait_bounds = match iter.peek() {
            Some(pair) if pair.as_rule() == Rule::trait_bounds => iter.next(),
            _ => None,
        };

        // create the type arguments
        let generic_type_arguments = check!(
            TypeParameter::parse_from_type_params_and_where_clause(
                generic_type_arguments,
                trait_bounds,
                config,
            ),
            vec!(),
            warnings,
            errors
        );
        let specific_type_arguments = specific_type_arguments
            .map(|x| {
                check!(
                    TypeInfo::parse_from_type_params(x, config),
                    vec!(),
                    warnings,
                    errors
                )
            })
            .unwrap_or_default();

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
                generic_type_arguments,
                generic_type_arguments_span,
                specific_type_arguments,
                specific_type_arguments_span,
                functions: fn_decls_buf,
                block_span,
            },
            warnings,
            errors,
        )
    }
}
