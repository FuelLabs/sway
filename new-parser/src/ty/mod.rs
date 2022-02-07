use crate::priv_prelude::*;

mod array;
mod tuple;

pub use array::*;
pub use tuple::*;

#[derive(Debug, Clone)]
pub enum Ty {
    Path {
        path: Path,
        generics_opt: Option<AngleBrackets<Punctuated<Ty, CommaToken>>>,
    },
    Tuple(TyTuple),
    Array(TyArray),
    Str {
        str_token: StrToken,
        length: SquareBrackets<Box<Expr>>,
    },
}

impl Spanned for Ty {
    fn span(&self) -> Span {
        match self {
            Ty::Path { path, generics_opt } => {
                match generics_opt {
                    Some(generics) => Span::join(path.span(), generics.span()),
                    None => path.span(),
                }
            },
            Ty::Tuple(ty_tuple) => ty_tuple.span(),
            Ty::Array(ty_array) => ty_array.span(),
            Ty::Str { str_token, length } => {
                Span::join(str_token.span(), length.span())
            },
        }
    }
}

pub fn ty() -> impl Parser<Output = Ty> + Clone {
    let ty_str = {
        str_token()
        .then_optional_whitespace()
        .then(square_brackets(padded(lazy(|| expr()).map(Box::new))))
        .map(|(str_token, length)| {
            Ty::Str { str_token, length }
        })
    };
    let path = {
        path()
        .then(optional_leading_whitespace(
            angle_brackets(punctuated(lazy(|| ty()), comma_token()))
            .optional()
        ))
        .map(|(path, generics_opt)| Ty::Path { path, generics_opt })
    };
    let tuple = {
        ty_tuple()
        .map(|ty_tuple| Ty::Tuple(ty_tuple))
    };
    let array = {
        ty_array()
        .map(|ty_array| Ty::Array(ty_array))
    };

    or! {
        ty_str,
        path,
        tuple,
        array,
    }
    .try_map_with_span(|ty_opt: Option<Ty>, span| {
        ty_opt.ok_or_else(|| ParseError::ExpectedType { span })
    })
}
