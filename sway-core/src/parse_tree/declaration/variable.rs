use crate::parser::Rule;
use crate::{
    error::{err, ok},
    ident,
    parse_tree::Expression,
    type_engine::TypeInfo,
    BuildConfig, CompileResult, Ident,
};

use pest::iterators::Pair;
use sway_types::span::Span;

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Ident,
    pub type_ascription: TypeInfo,
    pub type_ascription_span: Option<Span>,
    pub body: Expression, // will be codeblock variant
    pub is_mutable: bool,
}

/// This enum represents the possibilities of what can be placed
/// on the LHS of a variable declaration. Given this declaration:
///
/// ```ignore
/// let (a, b) = (1, 2);
/// ```
///
/// The LHS would translate to a `VariableDeclarationLHS::Tuple(...)`.
/// However, these objects are not public and do not exist outside
/// of variable declaration desugaring. They get consumed in the
/// `parse_from_pair` function below.
enum VariableDeclarationLHS {
    Name(LHSName),
    Tuple(LHSTuple),
}

struct LHSName {
    name: Ident,
    is_mutable: bool,
    span: Span,
}

struct LHSTuple {
    elems: Vec<VariableDeclarationLHS>,
    span: Span,
}

impl VariableDeclaration {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut var_decl_parts = pair.into_inner();
        let _let_keyword = var_decl_parts.next();
        let lhs = check!(
            VariableDeclarationLHS::parse_from_pair(
                var_decl_parts.next().expect("gaurenteed by grammar"),
                config
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut maybe_body = var_decl_parts.next().unwrap();
        let type_ascription = match maybe_body.as_rule() {
            Rule::type_ascription => {
                let type_asc = maybe_body.clone();
                maybe_body = var_decl_parts.next().unwrap();
                Some(type_asc)
            }
            _ => None,
        };
        let type_ascription_span = type_ascription.clone().map(|x| {
            Span::from_pest(
                x.into_inner().next().unwrap().as_span(),
                config.map(|x| x.path()),
            )
        });
        let type_ascription = match type_ascription {
            Some(ascription) => {
                let type_name = ascription.into_inner().next().unwrap();
                check!(
                    TypeInfo::parse_from_pair(type_name, config),
                    TypeInfo::Tuple(Vec::new()),
                    warnings,
                    errors
                )
            }
            _ => TypeInfo::Unknown,
        };
        let body_result = check!(
            Expression::parse_from_pair(maybe_body, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut var_decls = body_result.var_decls;
        var_decls.append(&mut check!(
            VariableDeclaration::desugar_to_decls(
                lhs,
                type_ascription,
                type_ascription_span,
                body_result.value,
                config,
            ),
            return err(warnings, errors),
            warnings,
            errors
        ));
        ok(var_decls, warnings, errors)
    }

    fn desugar_to_decls(
        lhs: VariableDeclarationLHS,
        type_ascription: TypeInfo,
        type_ascription_span: Option<Span>,
        body: Expression,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let decls = match lhs {
            VariableDeclarationLHS::Name(LHSName {
                name, is_mutable, ..
            }) => {
                vec![VariableDeclaration {
                    name,
                    body,
                    is_mutable,
                    type_ascription,
                    type_ascription_span,
                }]
            }
            VariableDeclarationLHS::Tuple(lhs_tuple) => {
                let name = ident::random_name(body.span(), config);
                let save_body_first = VariableDeclaration {
                    name: name.clone(),
                    type_ascription,
                    type_ascription_span,
                    body: body.clone(),
                    is_mutable: false,
                };
                let new_body = Expression::VariableExpression {
                    name,
                    span: body.span(),
                };
                let mut decls = vec![save_body_first];
                decls.append(&mut check!(
                    VariableDeclaration::desugar_to_decls_inner(
                        VariableDeclarationLHS::Tuple(lhs_tuple),
                        new_body
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                decls
            }
        };
        ok(decls, warnings, errors)
    }

    fn desugar_to_decls_inner(
        lhs: VariableDeclarationLHS,
        body: Expression,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let decls = match lhs {
            VariableDeclarationLHS::Name(LHSName {
                name, is_mutable, ..
            }) => {
                vec![VariableDeclaration {
                    name,
                    body,
                    is_mutable,
                    type_ascription: TypeInfo::Unknown,
                    type_ascription_span: None,
                }]
            }
            VariableDeclarationLHS::Tuple(LHSTuple { elems, span }) => {
                let mut decls = vec![];
                for (pos, elem) in elems.into_iter().enumerate() {
                    let new_body = Expression::TupleIndex {
                        prefix: Box::new(body.clone()),
                        index: pos,
                        index_span: elem.span(),
                        span: span.clone(),
                    };
                    decls.append(&mut check!(
                        VariableDeclaration::desugar_to_decls_inner(elem, new_body),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                decls
            }
        };
        ok(decls, warnings, errors)
    }
}

impl VariableDeclarationLHS {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        assert_eq!(pair.as_rule(), Rule::var_lhs);
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = Span::from_pest(pair.as_span(), config.map(|x| x.path()));
        let inner = pair.into_inner().next().expect("gaurenteed by grammar.");
        let lhs = match inner.as_rule() {
            Rule::var_name => {
                let mut parts = inner.into_inner();
                let maybe_mut_keyword = parts.next().unwrap();
                let is_mutable = maybe_mut_keyword.as_rule() == Rule::mut_keyword;
                let name_pair = if is_mutable {
                    parts.next().unwrap()
                } else {
                    maybe_mut_keyword
                };
                let name = check!(
                    ident::parse_from_pair(name_pair, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                VariableDeclarationLHS::Name(LHSName {
                    name,
                    is_mutable,
                    span,
                })
            }
            Rule::var_tuple => {
                let fields = inner.into_inner().collect::<Vec<_>>();
                let mut fields_buf = Vec::with_capacity(fields.len());
                for field in fields.into_iter() {
                    fields_buf.push(check!(
                        VariableDeclarationLHS::parse_from_pair(field, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                VariableDeclarationLHS::Tuple(LHSTuple {
                    elems: fields_buf,
                    span,
                })
            }
            a => unreachable!("Grammar should prevent this case from being {:?}", a),
        };
        ok(lhs, warnings, errors)
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            VariableDeclarationLHS::Name(LHSName { span, .. }) => span.clone(),
            VariableDeclarationLHS::Tuple(LHSTuple { span, .. }) => span.clone(),
        }
    }
}
