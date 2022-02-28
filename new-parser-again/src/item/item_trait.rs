use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemTrait {
    pub visibility: Option<PubToken>,
    pub trait_token: TraitToken,
    pub name: Ident,
    pub trait_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
}

impl Parse for ItemTrait {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTrait> {
        let visibility = parser.take();
        let trait_token = parser.parse()?;
        let name = parser.parse()?;
        let trait_items = parser.parse()?;
        Ok(ItemTrait { visibility, trait_token, name, trait_items })
    }
}

