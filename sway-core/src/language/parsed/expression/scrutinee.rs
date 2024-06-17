use crate::{
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::{CallPath, Literal},
    TypeInfo,
};

use sway_error::handler::ErrorEmitted;
use sway_types::{ident::Ident, span::Span, Spanned};

/// A [Scrutinee] is on the left-hand-side of a pattern, and dictates whether or
/// not a pattern will succeed at pattern matching and what, if any, elements will
/// need to be implemented in a desugared if expression.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Scrutinee {
    Or {
        elems: Vec<Scrutinee>,
        span: Span,
    },
    CatchAll {
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
    AmbiguousSingleIdent(Ident),
    StructScrutinee {
        struct_name: CallPath,
        fields: Vec<StructScrutineeField>,
        span: Span,
    },
    EnumScrutinee {
        call_path: CallPath,
        value: Box<Scrutinee>,
        span: Span,
    },
    Tuple {
        elems: Vec<Scrutinee>,
        span: Span,
    },
    // this is to handle parser recovery
    Error {
        spans: Box<[Span]>,
        err: ErrorEmitted,
    },
}

impl EqWithEngines for Scrutinee {}
impl PartialEqWithEngines for Scrutinee {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (
                Scrutinee::Or { elems, span },
                Scrutinee::Or {
                    elems: r_elems,
                    span: r_span,
                },
            ) => elems.eq(r_elems, ctx) && span.eq(r_span),
            (
                Scrutinee::Literal { value, span },
                Scrutinee::Literal {
                    value: r_value,
                    span: r_span,
                },
            ) => value.eq(r_value) && span.eq(r_span),
            (
                Scrutinee::Variable { name, span },
                Scrutinee::Variable {
                    name: r_name,
                    span: r_span,
                },
            ) => name.eq(r_name) && span.eq(r_span),
            (Scrutinee::AmbiguousSingleIdent(ident), Scrutinee::AmbiguousSingleIdent(r_ident)) => {
                ident.eq(r_ident)
            }
            (
                Scrutinee::StructScrutinee {
                    struct_name,
                    fields,
                    span,
                },
                Scrutinee::StructScrutinee {
                    struct_name: r_struct_name,
                    fields: r_fields,
                    span: r_span,
                },
            ) => {
                PartialEqWithEngines::eq(struct_name, r_struct_name, ctx)
                    && fields.eq(r_fields, ctx)
                    && span.eq(r_span)
            }
            (
                Scrutinee::EnumScrutinee {
                    call_path,
                    value,
                    span,
                },
                Scrutinee::EnumScrutinee {
                    call_path: r_call_path,
                    value: r_value,
                    span: r_span,
                },
            ) => {
                PartialEqWithEngines::eq(call_path, r_call_path, ctx)
                    && value.eq(r_value, ctx)
                    && span.eq(r_span)
            }
            (
                Scrutinee::Tuple { elems, span },
                Scrutinee::Tuple {
                    elems: r_elems,
                    span: r_span,
                },
            ) => elems.eq(r_elems, ctx) && span.eq(r_span),
            (
                Scrutinee::Error { spans, err },
                Scrutinee::Error {
                    spans: r_spans,
                    err: r_err,
                },
            ) => spans.eq(r_spans) && err.eq(r_err),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum StructScrutineeField {
    Rest {
        span: Span,
    },
    Field {
        field: Ident,
        scrutinee: Option<Scrutinee>,
        span: Span,
    },
}

impl EqWithEngines for StructScrutineeField {}
impl PartialEqWithEngines for StructScrutineeField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (StructScrutineeField::Rest { span }, StructScrutineeField::Rest { span: r_span }) => {
                span == r_span
            }
            (
                StructScrutineeField::Field {
                    field,
                    scrutinee,
                    span,
                },
                StructScrutineeField::Field {
                    field: r_field,
                    scrutinee: r_scrutinee,
                    span: r_span,
                },
            ) => field.eq(r_field) && scrutinee.eq(r_scrutinee, ctx) && span.eq(r_span),
            _ => false,
        }
    }
}

impl Spanned for Scrutinee {
    fn span(&self) -> Span {
        match self {
            Scrutinee::Or { span, .. } => span.clone(),
            Scrutinee::CatchAll { span } => span.clone(),
            Scrutinee::Literal { span, .. } => span.clone(),
            Scrutinee::Variable { span, .. } => span.clone(),
            Scrutinee::AmbiguousSingleIdent(ident) => ident.span(),
            Scrutinee::StructScrutinee { span, .. } => span.clone(),
            Scrutinee::EnumScrutinee { span, .. } => span.clone(),
            Scrutinee::Tuple { span, .. } => span.clone(),
            Scrutinee::Error { spans, .. } => spans
                .iter()
                .cloned()
                .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
                .unwrap(),
        }
    }
}

impl Scrutinee {
    /// Given some `Scrutinee` `self`, analyze `self` and return all instances
    /// of possible dependencies. A "possible dependency" is a `Scrutinee` that
    /// resolves to one or more `TypeInfo::Custom`.
    ///
    /// For example, this `Scrutinee`:
    ///
    /// ```ignore
    /// Scrutinee::EnumScrutinee {
    ///   call_path: CallPath {
    ///     prefixes: ["Data"]
    ///     suffix: "A"
    ///   },
    ///   value: Scrutinee::StructScrutinee {
    ///     struct_name: "Foo",
    ///     fields: [
    ///         StructScrutineeField {
    ///             scrutinee: Scrutinee::StructScrutinee {
    ///                 struct_name: "Bar",
    ///                 fields: [
    ///                     StructScrutineeField {
    ///                         scrutinee: Scrutinee::Literal { .. },
    ///                         ..
    ///                     }
    ///                 ],
    ///                 ..
    ///             },
    ///             ..
    ///         }
    ///     ],
    ///     ..
    ///   }
    ///   ..
    /// }
    /// ```
    ///
    /// would resolve to this list of approximate `TypeInfo` dependencies:
    ///
    /// ```ignore
    /// [
    ///     TypeInfo::Custom {
    ///         name: "Data",
    ///         ..
    ///     },
    ///     TypeInfo::Custom {
    ///         name: "Foo",
    ///         ..
    ///     },
    ///     TypeInfo::Custom {
    ///         name: "Bar",
    ///         ..
    ///     },
    /// ]
    /// ```
    pub(crate) fn gather_approximate_typeinfo_dependencies(&self) -> Vec<TypeInfo> {
        match self {
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                let name = vec![TypeInfo::Custom {
                    qualified_call_path: struct_name.clone().into(),
                    type_arguments: None,
                    root_type_id: None,
                }];
                let fields = fields
                    .iter()
                    .flat_map(|f| match f {
                        StructScrutineeField::Field {
                            scrutinee: Some(scrutinee),
                            ..
                        } => scrutinee.gather_approximate_typeinfo_dependencies(),
                        _ => vec![],
                    })
                    .collect::<Vec<TypeInfo>>();
                [name, fields].concat()
            }
            Scrutinee::EnumScrutinee {
                call_path, value, ..
            } => {
                let enum_name = call_path.prefixes.last().unwrap_or(&call_path.suffix);
                let name = vec![TypeInfo::Custom {
                    qualified_call_path: enum_name.clone().into(),
                    type_arguments: None,
                    root_type_id: None,
                }];
                let value = value.gather_approximate_typeinfo_dependencies();
                [name, value].concat()
            }
            Scrutinee::Tuple { elems, .. } | Scrutinee::Or { elems, .. } => elems
                .iter()
                .flat_map(|scrutinee| scrutinee.gather_approximate_typeinfo_dependencies())
                .collect::<Vec<TypeInfo>>(),
            Scrutinee::Literal { .. }
            | Scrutinee::CatchAll { .. }
            | Scrutinee::AmbiguousSingleIdent(..)
            | Scrutinee::Variable { .. }
            | Scrutinee::Error { .. } => {
                vec![]
            }
        }
    }
}
