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

impl Spanned for Assignable {
    fn span(&self) -> Span {
        match self {
            Assignable::Var(name) => name.span(),
            Assignable::Index { target, arg } => {
                Span::join(target.span(), arg.span())
            },
            Assignable::FieldProjection { target, name, .. } => {
                Span::join(target.span(), name.span())
            },
        }
    }
}

pub fn assignable() -> impl Parser<Output = Assignable> + Clone {
    enum Kind {
        Index(SquareBrackets<Box<Expr>>),
        FieldProjection {
            dot_token: DotToken,
            name: Ident,
        },
    }

    let kind = {
        let index = {
            square_brackets(padded(lazy(|| expr()).map(Box::new)))
            .map(Kind::Index)
        };
        let field_projection = {
            dot_token()
            .then_optional_whitespace()
            .then(ident())
            .map(|(dot_token, name)| {
                Kind::FieldProjection {
                    dot_token,
                    name,
                }
            })
        };

        or! {
            field_projection,
            index,
        }
    };

    ident()
    .then(optional_leading_whitespace(kind).while_some())
    .map(|(base_name, kinds)| {
        let mut target = Assignable::Var(base_name);
        for kind in kinds {
            match kind {
                Kind::Index(arg) => {
                    target = Assignable::Index {
                        target: Box::new(target),
                        arg,
                    };
                },
                Kind::FieldProjection { dot_token, name } => {
                    target = Assignable::FieldProjection {
                        target: Box::new(target),
                        dot_token,
                        name,
                    };
                },
            }
        }
        target
    })
}

