use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Assignable {
    Var(Ident),
    Index {
        target: Box<Assignable>,
        arg: SquareBrackets<Box<Expr>>,
    },
    FieldProjection {
        target: Box<Assignable>,
        dot_token: DotToken,
        name: Ident,
    },
    TupleFieldProjection {
        target: Box<Assignable>,
        dot_token: DotToken,
        field: BigUint,
        field_span: Span,
    },
}

impl Spanned for Assignable {
    fn span(&self) -> Span {
        match self {
            Assignable::Var(name) => name.span(),
            Assignable::Index { target, arg } => Span::join(target.span(), arg.span()),
            Assignable::FieldProjection { target, name, .. } => {
                Span::join(target.span(), name.span())
            }
            Assignable::TupleFieldProjection {
                target, field_span, ..
            } => Span::join(target.span(), field_span.clone()),
        }
    }
}
