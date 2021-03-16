use super::{FunctionDeclaration, TypeParameter, VarName};
use crate::{error::*, parser::Rule, types::TypeInfo};
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub(crate) struct ImplTrait<'sc> {
    pub(crate) trait_name: VarName<'sc>,
    pub(crate) type_implementing_for: TypeInfo<'sc>,
    pub(crate) type_arguments: Vec<TypeParameter<'sc>>,
    pub(crate) functions: Vec<FunctionDeclaration<'sc>>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span<'sc>,
    pub(crate) type_arguments_span: Span<'sc>,
}

/// An impl of methods without a trait
/// like `impl MyType { fn foo { .. } }`
#[derive(Debug, Clone)]
pub(crate) struct ImplSelf<'sc> {
    pub(crate) type_implementing_for: TypeInfo<'sc>,
    pub(crate) type_arguments: Vec<TypeParameter<'sc>>,
    pub(crate) functions: Vec<FunctionDeclaration<'sc>>,
    // the span of the whole impl trait and block
    pub(crate) block_span: Span<'sc>,
    pub(crate) type_arguments_span: Span<'sc>,
}

impl<'sc> ImplTrait<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let block_span = pair.as_span();
        let mut iter = pair.into_inner();
        let impl_keyword = iter.next().unwrap();
        assert_eq!(impl_keyword.as_str(), "impl");
        let trait_name = iter.next().unwrap();
        assert_eq!(trait_name.as_rule(), Rule::trait_name);
        let trait_name = eval!(
            VarName::parse_from_pair,
            warnings,
            errors,
            trait_name,
            return err(warnings, errors)
        );
        let mut iter = iter.peekable();
        let type_params_pair = if iter.peek().unwrap().as_rule() == Rule::type_params {
            iter.next()
        } else {
            None
        };

        let type_implementing_for = eval!(
            TypeInfo::parse_from_pair,
            warnings,
            errors,
            iter.next().unwrap(),
            return err(warnings, errors)
        );

        let where_clause_pair = if iter.peek().unwrap().as_rule() == Rule::trait_bounds {
            iter.next()
        } else {
            None
        };
        let type_arguments_span = match type_params_pair {
            Some(ref x) => x.as_span(),
            None => trait_name.span.clone(),
        };
        let type_arguments = match TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
        ) {
            CompileResult::Ok {
                errors: mut l_e,
                warnings: mut l_w,
                value,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                value
            }
            CompileResult::Err {
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                Vec::new()
            }
        };

        let mut fn_decls_buf = vec![];

        for pair in iter {
            fn_decls_buf.push(eval!(
                FunctionDeclaration::parse_from_pair,
                warnings,
                errors,
                pair,
                continue
            ));
        }

        ok(
            ImplTrait {
                trait_name,
                type_arguments,
                type_arguments_span,
                type_implementing_for,
                functions: fn_decls_buf,
                block_span,
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> ImplSelf<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let block_span = pair.as_span();
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
        let backup_span = type_pair.as_span();

        let type_implementing_for = eval!(
            TypeInfo::parse_from_pair,
            warnings,
            errors,
            type_pair,
            return err(warnings, errors)
        );

        let where_clause_pair = if iter.peek().unwrap().as_rule() == Rule::trait_bounds {
            iter.next()
        } else {
            None
        };
        let type_arguments_span = match type_params_pair {
            Some(ref x) => x.as_span(),
            None => backup_span,
        };
        let type_arguments = match TypeParameter::parse_from_type_params_and_where_clause(
            type_params_pair,
            where_clause_pair,
        ) {
            CompileResult::Ok {
                errors: mut l_e,
                warnings: mut l_w,
                value,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                value
            }
            CompileResult::Err {
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                Vec::new()
            }
        };

        let mut fn_decls_buf = vec![];

        for pair in iter {
            fn_decls_buf.push(eval!(
                FunctionDeclaration::parse_from_pair,
                warnings,
                errors,
                pair,
                continue
            ));
        }

        ok(
            ImplSelf {
                type_arguments,
                type_arguments_span,
                type_implementing_for,
                functions: fn_decls_buf,
                block_span,
            },
            warnings,
            errors,
        )
    }
}
