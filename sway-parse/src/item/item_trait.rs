use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemTrait {
    pub visibility: Option<PubToken>,
    pub trait_token: TraitToken,
    pub name: Ident,
    pub super_traits: Option<(ColonToken, Traits)>,
    pub trait_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
    pub trait_defs_opt: Option<Braces<Vec<ItemFn>>>,
}

impl ItemTrait {
    pub fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.trait_token.span(),
        };
        let end = match &self.trait_defs_opt {
            Some(trait_defs) => trait_defs.span(),
            None => self.trait_items.span(),
        };
        Span::join(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct Traits {
    pub prefix: PathType,
    pub suffixes: Vec<(AddToken, PathType)>,
}

impl Parse for ItemTrait {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTrait> {
        let visibility = parser.take();
        let trait_token = parser.parse()?;
        let name = parser.parse()?;
        let super_traits = match parser.take() {
            Some(colon_token) => {
                let traits = parser.parse()?;
                Some((colon_token, traits))
            }
            None => None,
        };
        let trait_items = parser.parse()?;
        let trait_defs_opt = Braces::try_parse(parser)?;
        Ok(ItemTrait {
            visibility,
            trait_token,
            name,
            super_traits,
            trait_items,
            trait_defs_opt,
        })
    }
}

impl Parse for Traits {
    fn parse(parser: &mut Parser) -> ParseResult<Traits> {
        let prefix = parser.parse()?;
        let mut suffixes = Vec::new();
        while let Some(add_token) = parser.take() {
            let suffix = parser.parse()?;
            suffixes.push((add_token, suffix));
        }
        let traits = Traits { prefix, suffixes };
        Ok(traits)
    }
}

impl Traits {
    pub fn span(&self) -> Span {
        match self.suffixes.last() {
            Some((_add_token, path_type)) => Span::join(self.prefix.span(), path_type.span()),
            None => self.prefix.span(),
        }
    }
}
