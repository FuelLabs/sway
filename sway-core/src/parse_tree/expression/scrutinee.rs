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
    pub scrutinee: Option<Scrutinee>,
    pub span: Span,
}

impl Scrutinee {
    pub fn span(&self) -> Span {
        match self {
            Scrutinee::CatchAll { span } => span.clone(),
            Scrutinee::Literal { span, .. } => span.clone(),
            Scrutinee::Variable { span, .. } => span.clone(),
            Scrutinee::StructScrutinee { span, .. } => span.clone(),
            Scrutinee::EnumScrutinee { span, .. } => span.clone(),
            Scrutinee::Tuple { span, .. } => span.clone(),
        }
    }

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
                    name: struct_name.clone(),
                    type_arguments: vec![],
                }];
                let fields = fields
                    .iter()
                    .flat_map(|StructScrutineeField { scrutinee, .. }| match scrutinee {
                        Some(scrutinee) => scrutinee.gather_approximate_typeinfo_dependencies(),
                        None => vec![],
                    })
                    .collect::<Vec<TypeInfo>>();
                vec![name, fields].concat()
            }
            Scrutinee::EnumScrutinee {
                call_path, value, ..
            } => {
                let enum_name = call_path.prefixes.last().unwrap_or(&call_path.suffix);
                let name = vec![TypeInfo::Custom {
                    name: enum_name.clone(),
                    type_arguments: vec![],
                }];
                let value = value.gather_approximate_typeinfo_dependencies();
                vec![name, value].concat()
            }
            Scrutinee::Tuple { elems, .. } => elems
                .iter()
                .flat_map(|scrutinee| scrutinee.gather_approximate_typeinfo_dependencies())
                .collect::<Vec<TypeInfo>>(),
            Scrutinee::Literal { .. } | Scrutinee::CatchAll { .. } | Scrutinee::Variable { .. } => {
                vec![]
            }
        }
    }
}
