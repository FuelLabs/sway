use std::{path::PathBuf, sync::Arc};

use crate::{
    error::{err, ok},
    parse_tree::ident,
    BuildConfig, CallPath, CompileError, CompileResult, Literal, Rule,
};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

/// A [Scrutinee] is on the left-hand-side of a pattern, and dictates whether or
/// not a pattern will succeed at pattern matching and what, if any, elements will
/// need to be implemented in a desugared if expression.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Scrutinee {
    Unit {
        span: Span,
    },
    Literal {
        value: Literal,
        span: Span,
    },
    Variable {
        name: Ident,
        span: Span,
    },
    StructScrutinee {
        struct_name: Ident,
        fields: Vec<StructScrutineeField>,
        span: Span,
    },
    EnumScrutinee {
        call_path: CallPath,
        variable_to_assign: Ident,
        span: Span,
    },
    Tuple {
        elems: Vec<Scrutinee>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct StructScrutineeField {
    pub field: Ident,
    pub scrutinee: Option<Scrutinee>,
    pub span: Span,
}

impl Scrutinee {
    pub fn span(&self) -> Span {
        match self {
            Scrutinee::Literal { span, .. } => span.clone(),
            Scrutinee::Unit { span } => span.clone(),
            Scrutinee::Variable { span, .. } => span.clone(),
            Scrutinee::StructScrutinee { span, .. } => span.clone(),
            Scrutinee::EnumScrutinee { span, .. } => span.clone(),
            Scrutinee::Tuple { span, .. } => span.clone(),
        }
    }

    pub fn parse_from_pair(pair: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut scrutinees = pair.into_inner();
        let scrutinee = scrutinees.next().unwrap();
        let scrutinee = check!(
            Scrutinee::parse_from_pair_inner(scrutinee, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(scrutinee, warnings, errors)
    }

    /// If this is an enum scrutinee, returns the name of the inner value that should be
    /// assigned to upon successful destructuring.
    /// Should only be used when destructuring enums via `if let`
    pub fn enum_variable_to_assign(&self) -> CompileResult<&Ident> {
        match self {
            Scrutinee::EnumScrutinee {
                variable_to_assign, ..
            } => ok(variable_to_assign, vec![], vec![]),
            _ => err(
                vec![],
                vec![CompileError::IfLetNonEnum { span: self.span() }],
            ),
        }
    }

    pub fn parse_from_pair_inner(
        scrutinee: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let span = Span::from_pest(scrutinee.as_span(), path.clone());
        let parsed = match scrutinee.as_rule() {
            Rule::literal_value => check!(
                Self::parse_from_pair_literal(scrutinee, config, span),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::ident => check!(
                Self::parse_from_pair_ident(scrutinee, config, span),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::struct_scrutinee => check!(
                Self::parse_from_pair_struct(scrutinee, config, span, path),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::enum_scrutinee => check!(
                Self::parse_from_pair_enum(scrutinee, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::tuple_scrutinee => check!(
                Self::parse_from_pair_tuple(scrutinee, config, span),
                return err(warnings, errors),
                warnings,
                errors
            ),
            a => {
                eprintln!(
                    "Unimplemented scrutinee: {:?} ({:?}) ({:?})",
                    a,
                    scrutinee.as_str(),
                    scrutinee.as_rule()
                );
                errors.push(CompileError::UnimplementedRule(
                    a,
                    Span::from_pest(scrutinee.as_span(), path.clone()),
                ));
                // construct unit expression for error recovery
                Scrutinee::Unit {
                    span: Span::from_pest(scrutinee.as_span(), path),
                }
            }
        };
        ok(parsed, warnings, errors)
    }

    fn parse_from_pair_literal(
        scrutinee: Pair<Rule>,
        config: Option<&BuildConfig>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let scrutinee = Literal::parse_from_pair(scrutinee, config)
            .map(|(value, span)| Scrutinee::Literal { value, span })
            .unwrap_or_else(&mut warnings, &mut errors, || Scrutinee::Unit {
                span: span.clone(),
            });
        ok(scrutinee, warnings, errors)
    }

    fn parse_from_pair_ident(
        scrutinee: Pair<Rule>,
        config: Option<&BuildConfig>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let scrutinee = ident::parse_from_pair(scrutinee, config)
            .map(|name| Scrutinee::Variable {
                name,
                span: span.clone(),
            })
            .unwrap_or_else(&mut warnings, &mut errors, || Scrutinee::Unit {
                span: span.clone(),
            });
        ok(scrutinee, warnings, errors)
    }

    fn parse_from_pair_struct(
        scrutinee: Pair<Rule>,
        config: Option<&BuildConfig>,
        span: Span,
        path: Option<Arc<PathBuf>>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut it = scrutinee.into_inner();
        let struct_name = it.next().unwrap();
        let struct_name = check!(
            ident::parse_from_pair(struct_name, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let fields = it.next().unwrap().into_inner().collect::<Vec<_>>();
        let mut fields_buf = vec![];
        for field in fields.iter() {
            let span = Span::from_pest(field.as_span(), path.clone());
            let mut field_parts = field.clone().into_inner();
            let name = field_parts.next().unwrap();
            let name = check!(
                ident::parse_from_pair(name, config),
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

        let scrutinee = Scrutinee::StructScrutinee {
            struct_name,
            fields: fields_buf,
            span,
        };
        ok(scrutinee, warnings, errors)
    }

    fn parse_from_pair_enum(pair: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = pair.as_span();
        let span = Span::from_pest(span, config.map(|x| x.path()));
        let mut iter = pair.into_inner();
        let enum_scrutinee_component_pair = iter.next().expect("guaranteed by grammar");
        let assignment_for_value_pair = iter.next().expect("guaranteed by grammar");
        let variable_to_assign = check!(
            ident::parse_from_pair(assignment_for_value_pair, config),
            return err(warnings, errors),
            warnings,
            errors
        );

        let call_path = check!(
            CallPath::parse_from_pair(enum_scrutinee_component_pair, config),
            return err(warnings, errors),
            warnings,
            errors
        );

        ok(
            Scrutinee::EnumScrutinee {
                call_path,
                span,
                variable_to_assign,
            },
            warnings,
            errors,
        )
    }

    fn parse_from_pair_tuple(
        scrutinee: Pair<Rule>,
        config: Option<&BuildConfig>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let parts = scrutinee.into_inner();
        let mut elems = vec![];
        for part in parts {
            elems.push(check!(
                Scrutinee::parse_from_pair(part, config),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        let scrutinee = Scrutinee::Tuple { elems, span };
        ok(scrutinee, warnings, errors)
    }
}
