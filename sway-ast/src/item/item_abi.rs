use crate::{priv_prelude::*, ItemTraitItem};

#[derive(Clone, Debug, Serialize)]
pub struct ItemAbi {
    pub abi_token: AbiToken,
    pub name: Ident,
    pub super_traits: Option<(ColonToken, Traits)>,
    pub abi_items: Braces<Vec<(Annotated<ItemTraitItem>, SemicolonToken)>>,
    pub abi_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>>,
}

impl Spanned for ItemAbi {
    fn span(&self) -> Span {
        let start = self.abi_token.span();
        let end = match &self.abi_defs_opt {
            Some(abi_defs) => abi_defs.span(),
            None => self.abi_items.span(),
        };
        Span::join(start, end)
    }
}
