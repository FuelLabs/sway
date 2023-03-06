use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum ItemImplItem {
    Fn(ItemFn),
    Const(ItemConst),
}

#[derive(Clone, Debug)]
pub struct ItemImpl {
    pub impl_token: ImplToken,
    pub generic_params_opt: Option<GenericParams>,
    pub trait_opt: Option<(PathType, ForToken)>,
    pub ty: Ty,
    pub where_clause_opt: Option<WhereClause>,
    pub contents: Braces<Vec<Annotated<ItemImplItem>>>,
}

impl Spanned for ItemImpl {
    fn span(&self) -> Span {
        Span::join(self.impl_token.span(), self.contents.span())
    }
}

impl Spanned for ItemImplItem {
    fn span(&self) -> Span {
        match self {
            ItemImplItem::Fn(fn_decl) => fn_decl.span(),
            ItemImplItem::Const(const_decl) => const_decl.span(),
        }
    }
}
