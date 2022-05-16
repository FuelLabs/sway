use crate::priv_prelude::*;

pub trait Spanned {
    fn span(&self) -> Span;
}
