use crate::priv_prelude::*;

mod array;
mod tuple;

pub use array::*;
pub use tuple::*;

pub enum Ty {
    Path {
        path: Path,
    },
    Tuple(TyTuple),
    Array(TyArray),
}

impl Spanned for Ty {
    fn span(&self) -> Span {
        match self {
            Ty::Path { path } => path.span(),
            Ty::Tuple(ty_tuple) => ty_tuple.span(),
            Ty::Array(ty_array) => ty_array.span(),
        }
    }
}

pub fn ty() -> impl Parser<Output = Ty> + Clone {
    let path = {
        path()
        .map(|path| Ty::Path { path })
    };
    let tuple = {
        ty_tuple()
        .map(|ty_tuple| Ty::Tuple(ty_tuple))
    };
    let array = {
        ty_array()
        .map(|ty_array| Ty::Array(ty_array))
    };

    path
    .or(tuple)
    .or(array)
}
