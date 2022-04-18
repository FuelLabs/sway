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
}

impl Assignable {
    pub fn span(&self) -> Span {
        match self {
            Assignable::Var(name) => name.span().clone(),
            Assignable::Index { target, arg } => Span::join(target.span(), arg.span()),
            Assignable::FieldProjection { target, name, .. } => {
                Span::join(target.span(), name.span().clone())
            }
        }
    }
}
