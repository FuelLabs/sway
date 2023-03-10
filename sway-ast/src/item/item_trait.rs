use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub enum ItemTraitItem {
    Fn(FnSignature),
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemTrait {
    pub visibility: Option<PubToken>,
    pub trait_token: TraitToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub where_clause_opt: Option<WhereClause>,
    pub super_traits: Option<(ColonToken, Traits)>,
    pub trait_items: Braces<Vec<(Annotated<ItemTraitItem>, SemicolonToken)>>,
    pub trait_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>>,
}

impl Spanned for ItemTrait {
    fn span(&self) -> Span {
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

impl Spanned for ItemTraitItem {
    fn span(&self) -> Span {
        match self {
            ItemTraitItem::Fn(fn_decl) => fn_decl.span(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Traits {
    pub prefix: PathType,
    pub suffixes: Vec<(AddToken, PathType)>,
}

impl Spanned for Traits {
    fn span(&self) -> Span {
        match self.suffixes.last() {
            Some((_add_token, path_type)) => Span::join(self.prefix.span(), path_type.span()),
            None => self.prefix.span(),
        }
    }
}
