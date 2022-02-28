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

