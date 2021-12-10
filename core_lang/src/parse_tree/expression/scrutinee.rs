use crate::{
    error::{err, ok},
    BuildConfig, CompileError, CompileResult, Ident, Literal, Rule, Span,
};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum Scrutinee<'sc> {
    Unit {
        span: Span<'sc>,
    },
    Literal {
        value: Literal<'sc>,
        span: Span<'sc>,
    },
    Variable {
        name: Ident<'sc>,
        span: Span<'sc>,
    },
    StructScrutinee {
        struct_name: Ident<'sc>,
        fields: Vec<StructScrutineeField<'sc>>,
        span: Span<'sc>,
    },
}

#[derive(Debug, Clone)]
pub struct StructScrutineeField<'sc> {
    pub field: Ident<'sc>,
    pub scrutinee: Option<Scrutinee<'sc>>,
    pub span: Span<'sc>,
}

impl<'sc> Scrutinee<'sc> {
    pub fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut scrutinees = pair.into_inner();
        let scrutinee = scrutinees.next().unwrap();
        let scrutinee = check!(
            Scrutinee::parse_from_pair_inner(scrutinee.clone(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(scrutinee, warnings, errors)
    }

    pub fn parse_from_pair_inner(
        scrutinee: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let span = Span {
            span: scrutinee.as_span(),
            path: path.clone(),
        };
        let parsed = match scrutinee.as_rule() {
            Rule::literal_value => Literal::parse_from_pair(scrutinee.clone(), config)
                .map(|(value, span)| Scrutinee::Literal { value, span })
                .unwrap_or_else(&mut warnings, &mut errors, || Scrutinee::Unit {
                    span: span.clone(),
                }),
            Rule::ident => Ident::parse_from_pair(scrutinee.clone(), config)
                .map(|name| Scrutinee::Variable {
                    name,
                    span: span.clone(),
                })
                .unwrap_or_else(&mut warnings, &mut errors, || Scrutinee::Unit {
                    span: span.clone(),
                }),
            Rule::struct_scrutinee => {
                /*
                let mut var_decl_parts = decl_inner.into_inner();
                let _let_keyword = var_decl_parts.next();
                let maybe_mut_keyword = var_decl_parts.next().unwrap();
                let is_mutable = maybe_mut_keyword.as_rule() == Rule::mut_keyword;
                let name_pair = if is_mutable {
                    var_decl_parts.next().unwrap()
                } else {
                    maybe_mut_keyword
                };
                let mut maybe_body = var_decl_parts.next().unwrap();
                let type_ascription = match maybe_body.as_rule() {
                    Rule::type_ascription => {
                        let type_asc = maybe_body.clone();
                        maybe_body = var_decl_parts.next().unwrap();
                        Some(type_asc)
                    }
                    _ => None,
                };
                let type_ascription_span = type_ascription
                    .clone()
                    .map(|x| x.into_inner().next().unwrap().as_span());
                let type_ascription = if let Some(ascription) = type_ascription {
                    let type_name = ascription.into_inner().next().unwrap();
                    check!(
                        TypeInfo::parse_from_pair(type_name, config),
                        TypeInfo::Unit,
                        warnings,
                        errors
                    )
                } else {
                    TypeInfo::Unknown
                };
                let body = check!(
                    Expression::parse_from_pair(maybe_body, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                Declaration::VariableDeclaration(VariableDeclaration {
                    name: check!(
                        Ident::parse_from_pair(name_pair, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                    body,
                    is_mutable,
                    type_ascription,
                    type_ascription_span: type_ascription_span.map(|type_ascription_span| Span {
                        span: type_ascription_span,
                        path: config.clone().map(|x| x.path()),
                    }),
                })
                */
                let mut it = scrutinee.into_inner();
                let struct_name = it.next().unwrap();
                let struct_name = check!(
                    Ident::parse_from_pair(struct_name, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let fields = it.next().unwrap().into_inner().collect::<Vec<_>>();
                let mut fields_buf = vec![];
                for field in fields.iter() {
                    let span = Span {
                        span: field.as_span(),
                        path: path.clone(),
                    };
                    let mut field_parts = field.clone().into_inner();
                    let name = field_parts.next().unwrap();
                    let name = check!(
                        Ident::parse_from_pair(name, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let maybe_field_scrutinee = field_parts.next();
                    let field_scrutinee = match maybe_field_scrutinee {
                        Some(field_scrutinee) => match field_scrutinee.as_rule() {
                            Rule::field_scrutinee => {
                                let field_scrutinee = field_scrutinee.into_inner().next().unwrap();
                                let field_scrutinee = check!(
                                    Scrutinee::parse_from_pair(field_scrutinee, config),
                                    Scrutinee::Unit { span: span.clone() },
                                    warnings,
                                    errors
                                );
                                Some(field_scrutinee)
                            }
                            _ => None,
                        },
                        None => None,
                    };
                    fields_buf.push(StructScrutineeField {
                        field: name,
                        scrutinee: field_scrutinee,
                        span,
                    });
                }

                Scrutinee::StructScrutinee {
                    struct_name,
                    fields: fields_buf,
                    span,
                }
            }
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    scrutinee.as_str(),
                    scrutinee.as_rule()
                );
                errors.push(CompileError::UnimplementedRule(
                    a,
                    Span {
                        span: scrutinee.as_span(),
                        path: path.clone(),
                    },
                ));
                // construct unit expression for error recovery
                Scrutinee::Unit {
                    span: Span {
                        span: scrutinee.as_span(),
                        path,
                    },
                }
            }
        };
        ok(parsed, warnings, errors)
    }
}
