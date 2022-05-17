use crate::{CallPath, Literal, TypeInfo};

use sway_types::{ident::Ident, span::Span};

/// A [Scrutinee] is on the left-hand-side of a pattern, and dictates whether or
/// not a pattern will succeed at pattern matching and what, if any, elements will
/// need to be implemented in a desugared if expression.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Scrutinee {
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
    StructScrutinee {
        struct_name: Ident,
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
}

#[derive(Debug, Clone)]
pub struct StructScrutineeField {
    pub field: Ident,
    pub(crate) scrutinee: Option<Scrutinee>,
    pub(crate) span: Span,
}

impl Scrutinee {
    pub fn span(&self) -> Span {
        match self {
            Scrutinee::Literal { span, .. } => span.clone(),
            Scrutinee::Variable { span, .. } => span.clone(),
            Scrutinee::StructScrutinee { span, .. } => span.clone(),
            Scrutinee::EnumScrutinee { span, .. } => span.clone(),
            Scrutinee::Tuple { span, .. } => span.clone(),
            Scrutinee::CatchAll { span } => span.clone(),
        }
    }

    pub(crate) fn gather_approximate_typeinfo(&self) -> Vec<TypeInfo> {
        match self {
            Scrutinee::Literal { value, .. } => vec![value.to_typeinfo()],
            Scrutinee::Variable { .. } => vec![TypeInfo::Unknown],
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                let name = vec![TypeInfo::Custom {
                    name: struct_name.clone(),
                    type_arguments: vec![],
                }];
                let fields = fields
                    .iter()
                    .flat_map(|StructScrutineeField { scrutinee, .. }| match scrutinee {
                        Some(scrutinee) => scrutinee.gather_approximate_typeinfo(),
                        None => vec![],
                    })
                    .collect::<Vec<TypeInfo>>();
                vec![name, fields].concat()
            }
            Scrutinee::EnumScrutinee { call_path, .. } => vec![TypeInfo::Custom {
                name: call_path.prefixes.last().unwrap().clone(),
                type_arguments: vec![],
            }],
            Scrutinee::Tuple { elems, .. } => elems
                .iter()
                .flat_map(|scrutinee| scrutinee.gather_approximate_typeinfo())
                .collect::<Vec<TypeInfo>>(),
            Scrutinee::CatchAll { .. } => vec![TypeInfo::Unknown],
        }
    }
}
