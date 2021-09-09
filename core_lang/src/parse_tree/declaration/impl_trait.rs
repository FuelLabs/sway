use super::{FunctionDeclaration, TypeParameter};
use crate::build_config::BuildConfig;
use crate::parse_tree::CallPath;
use crate::span::Span;
use crate::{error::*, parser::Rule, types::TypeInfo};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImplTrait<'sc> {
    pub(crate) trait_name: CallPath<'sc>,
    pub(crate) type_implementing_for: TypeInfo<'sc>,
    pub(crate) type_implementing_for_span: Span<'sc>,
    pub(crate) type_arguments: Vec<TypeParameter<'sc>>,
    pub functions: Vec<FunctionDeclaration<'sc>>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span<'sc>,
    pub(crate) type_arguments_span: Span<'sc>,
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub struct ImplSelf<'sc> {
    pub(crate) type_implementing_for: TypeInfo<'sc>,
    pub(crate) type_arguments: Vec<TypeParameter<'sc>>,
    pub functions: Vec<FunctionDeclaration<'sc>>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span<'sc>,
    pub(crate) type_arguments_span: Span<'sc>,
    pub(crate) type_name_span: Span<'sc>,
}

impl<'sc> ImplTrait<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.clone().map(|c| c.dir_of_code);
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
        assert_eq!(trait_name.as_rule(), Rule::trait_name);
        let trait_name = eval2!(
            CallPath::parse_from_pair,
            warnings,
            errors,
            trait_name,
            config.clone(),
            return err(warnings, errors)
        );
        let mut iter = iter.peekable();
        let type_params_pair = if iter.peek().unwrap().as_rule() == Rule::type_params {
            iter.next()
        } else {
            None
        };

        let type_implementing_for_pair = iter.next().expect("guaranteed by grammar");
        let type_implementing_for_span = Span {
            span: type_implementing_for_pair.as_span(),
            path: path.clone(),
        };
        let type_implementing_for = eval2!(
            TypeInfo::parse_from_pair,
            warnings,
            errors,
            type_implementing_for_pair,
            config.clone(),
            return err(warnings, errors)
        );

        let where_clause_pair = if iter.peek().unwrap().as_rule() == Rule::trait_bounds {
            iter.next()
        } else {
            None
        };
        let type_arguments_span = match type_params_pair {
            Some(ref x) => Span {
                span: x.as_span(),
                path: path.clone(),
            },
            None => trait_name.span(),
        };
        let type_arguments = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
            config.clone(),
        )
        .unwrap_or_else(&mut warnings, &mut errors, || Vec::new());

        let mut fn_decls_buf = vec![];

        for pair in iter {
            fn_decls_buf.push(eval2!(
                FunctionDeclaration::parse_from_pair,
                warnings,
                errors,
                pair,
                config.clone(),
                continue
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

impl<'sc> ImplSelf<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.clone().map(|c| c.dir_of_code);
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let block_span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut iter = pair.into_inner();
        let impl_keyword = iter.next().unwrap();
        assert_eq!(impl_keyword.as_str(), "impl");
        let mut iter = iter.peekable();
        let type_params_pair = if iter.peek().unwrap().as_rule() == Rule::type_params {
            iter.next()
        } else {
            None
        };
        let type_pair = iter.next().unwrap();
        let type_name_span = Span {
            span: type_pair.as_span(),
            path: path.clone(),
        };

        let type_implementing_for = eval2!(
            TypeInfo::parse_from_pair,
            warnings,
            errors,
            type_pair,
            config.clone(),
            return err(warnings, errors)
        );

        let where_clause_pair = match iter.peek() {
            Some(pair) if pair.as_rule() == Rule::trait_bounds => iter.next(),
            _ => None,
        };
        let type_arguments_span = match type_params_pair {
            Some(ref x) => Span {
                span: x.as_span(),
                path: path.clone(),
            },
            None => type_name_span.clone(),
        };
        let type_arguments = TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
            config.clone(),
        )
        .unwrap_or_else(&mut warnings, &mut errors, || Vec::new());

        let mut fn_decls_buf = vec![];

        for pair in iter {
            fn_decls_buf.push(eval2!(
                FunctionDeclaration::parse_from_pair,
                warnings,
                errors,
                pair,
                config.clone(),
                continue
            ));
        }

        ok(
            ImplSelf {
                type_arguments,
                type_arguments_span,
                type_implementing_for,
                functions: fn_decls_buf,
                type_name_span,
                block_span,
            },
            warnings,
            errors,
        )
    }
}
