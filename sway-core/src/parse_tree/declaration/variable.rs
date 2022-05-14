use crate::{
    error::{err, ok},
    ident,
    parse_tree::Expression,
    type_engine::TypeInfo,
    BuildConfig, CompileResult, Ident,
};

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
    pub(crate) fn span(&self) -> Span {
        match self {
            VariableDeclarationLHS::Name(LHSName { span, .. }) => span.clone(),
            VariableDeclarationLHS::Tuple(LHSTuple { span, .. }) => span.clone(),
        }
    }
}
