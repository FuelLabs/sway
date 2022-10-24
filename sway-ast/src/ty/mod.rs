use crate::priv_prelude::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Ty {
    Path(PathType),
    Tuple(Parens<TyTupleDescriptor>),
    Array(SquareBrackets<TyArrayDescriptor>),
    Str {
        str_token: StrToken,
        length:    SquareBrackets<Box<Expr>>,
    },
    Infer {
        underscore_token: UnderscoreToken,
    },
}

impl Spanned for Ty {
    fn span(&self) -> Span {
        match self {
            Ty::Path(path_type) => path_type.span(),
            Ty::Tuple(tuple_type) => tuple_type.span(),
            Ty::Array(array_type) => array_type.span(),
            Ty::Str { str_token, length } => Span::join(str_token.span(), length.span()),
            Ty::Infer { underscore_token } => underscore_token.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TyTupleDescriptor {
    Nil,
    Cons {
        head:        Box<Ty>,
        comma_token: CommaToken,
        tail:        Punctuated<Ty, CommaToken>,
    },
}

impl TyTupleDescriptor {
    pub fn to_tys(self) -> Vec<Ty> {
        match self {
            TyTupleDescriptor::Nil => vec![],
            TyTupleDescriptor::Cons { head, tail, .. } => {
                let mut tys = vec![*head];
                for ty in tail.into_iter() {
                    tys.push(ty);
                }
                tys
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TyArrayDescriptor {
    pub ty:              Box<Ty>,
    pub semicolon_token: SemicolonToken,
    pub length:          Box<Expr>,
}
