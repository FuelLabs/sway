use crate::priv_prelude::*;

/// Left-hand side of an assignment.
#[derive(Clone, Debug, Serialize)]
pub enum Assignable {
    /// A single variable or a path to a part of an aggregate.
    /// E.g.:
    ///  - `my_variable`
    ///  - `array[0].field.x.1`
    /// Note that within the path, we cannot have dereferencing
    /// (except, of course, in expressions inside of array index operator).
    /// This is guaranteed by the grammar.
    /// E.g., an expression like this is not allowed by the grammar:
    ///  `my_struct.*expr`
    ElementAccess(ElementAccess),
    /// Dereferencing of an arbitrary reference expression.
    /// E.g.:
    ///  - *my_ref
    ///  - **if x > 0 { &mut &mut a } else { &mut &mut b }
    Deref {
        star_token: StarToken,
        expr: Box<Expr>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub enum ElementAccess {
    Var(Ident),
    Index {
        target: Box<ElementAccess>,
        arg: SquareBrackets<Box<Expr>>,
    },
    FieldProjection {
        target: Box<ElementAccess>,
        dot_token: DotToken,
        name: Ident,
    },
    TupleFieldProjection {
        target: Box<ElementAccess>,
        dot_token: DotToken,
        field: BigUint,
        field_span: Span,
    },
}

impl Spanned for Assignable {
    fn span(&self) -> Span {
        match self {
            Assignable::ElementAccess(element_access) => element_access.span(),
            Assignable::Deref { star_token, expr } => Span::join(star_token.span(), &expr.span()),
        }
    }
}

impl Spanned for ElementAccess {
    fn span(&self) -> Span {
        match self {
            ElementAccess::Var(name) => name.span(),
            ElementAccess::Index { target, arg } => Span::join(target.span(), &arg.span()),
            ElementAccess::FieldProjection { target, name, .. } => {
                Span::join(target.span(), &name.span())
            }
            ElementAccess::TupleFieldProjection {
                target, field_span, ..
            } => Span::join(target.span(), field_span),
        }
    }
}
