use crate::priv_prelude::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize)]
pub enum Ty {
    Path(PathType),
    Tuple(Parens<TyTupleDescriptor>),
    Array(SquareBrackets<TyArrayDescriptor>),
    StringSlice(StrToken),
    StringArray {
        str_token: StrToken,
        length: SquareBrackets<Box<Expr>>,
    },
    Infer {
        underscore_token: UnderscoreToken,
    },
    Ptr {
        ptr_token: PtrToken,
        ty: SquareBrackets<Box<Ty>>,
    },
    Slice {
        slice_token: Option<SliceToken>,
        ty: SquareBrackets<Box<Ty>>,
    },
    Ref {
        ampersand_token: AmpersandToken,
        mut_token: Option<MutToken>,
        ty: Box<Ty>,
    },
    Never {
        bang_token: BangToken,
    },
    Expr(Box<Expr>),
}

impl Spanned for Ty {
    fn span(&self) -> Span {
        match self {
            Ty::Path(path_type) => path_type.span(),
            Ty::Tuple(tuple_type) => tuple_type.span(),
            Ty::Array(array_type) => array_type.span(),
            Ty::StringSlice(str_token) => str_token.span(),
            Ty::StringArray { str_token, length } => Span::join(str_token.span(), &length.span()),
            Ty::Infer { underscore_token } => underscore_token.span(),
            Ty::Ptr { ptr_token, ty } => Span::join(ptr_token.span(), &ty.span()),
            Ty::Slice { slice_token, ty } => {
                let span = slice_token
                    .as_ref()
                    .map(|s| Span::join(s.span(), &ty.span()));
                span.unwrap_or_else(|| ty.span())
            }
            Ty::Ref {
                ampersand_token,
                mut_token: _,
                ty,
            } => Span::join(ampersand_token.span(), &ty.span()),
            Ty::Never { bang_token } => bang_token.span(),
            Ty::Expr(expr) => expr.span(),
        }
    }
}

impl Ty {
    pub fn name_span(&self) -> Option<Span> {
        if let Ty::Path(path_type) = self {
            Some(path_type.last_segment().name.span())
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum TyTupleDescriptor {
    Nil,
    Cons {
        head: Box<Ty>,
        comma_token: CommaToken,
        tail: Punctuated<Ty, CommaToken>,
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

#[derive(Clone, Debug, Serialize)]
pub struct TyArrayDescriptor {
    pub ty: Box<Ty>,
    pub semicolon_token: SemicolonToken,
    pub length: Box<Expr>,
}
