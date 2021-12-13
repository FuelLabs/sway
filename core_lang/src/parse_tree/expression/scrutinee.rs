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
