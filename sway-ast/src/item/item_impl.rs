use crate::{priv_prelude::*, FnArgs};

/// Denotes to what kind of an item an [ItemImplItem] belongs.
/// This enum is used mostly for reporting use cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplItemParent {
    Contract,
    // Currently we don't have cases that need further distinction.
    // Add other specific items like enum, struct, etc. when needed.
    Other,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize)]
pub enum ItemImplItem {
    Fn(ItemFn),
    Const(ItemConst),
    Type(TraitType),
}

impl ItemImplItem {
    /// [ItemImplItem]'s friendly name string used for various reportings.
    pub fn friendly_name(&self, parent: ImplItemParent) -> &'static str {
        use ItemImplItem::*;
        match self {
            Fn(item_fn) => match item_fn.fn_signature.arguments.inner {
                FnArgs::Static(_) => match parent {
                    ImplItemParent::Contract => "contract method",
                    ImplItemParent::Other => "associated function",
                },
                FnArgs::NonStatic { .. } => "method",
            },
            Const(..) => "associated constant",
            Type(..) => "associated type",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
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
        Span::join(self.impl_token.span(), &self.contents.span())
    }
}

impl Spanned for ItemImplItem {
    fn span(&self) -> Span {
        match self {
            ItemImplItem::Fn(fn_decl) => fn_decl.span(),
            ItemImplItem::Const(const_decl) => const_decl.span(),
            ItemImplItem::Type(type_decl) => type_decl.span(),
        }
    }
}
