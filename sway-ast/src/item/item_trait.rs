use sway_error::handler::ErrorEmitted;

use crate::priv_prelude::*;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize)]
pub enum ItemTraitItem {
    Fn(FnSignature, Option<SemicolonToken>),
    Const(ItemConst, Option<SemicolonToken>),
    Type(TraitType, Option<SemicolonToken>),
    // to handle parser recovery: Error represents an incomplete trait item
    Error(Box<[Span]>, #[serde(skip_serializing)] ErrorEmitted),
}

impl ItemTraitItem {
    /// [ItemTraitItem]'s friendly name string used for various reportings.
    pub fn friendly_name(&self) -> &'static str {
        use ItemTraitItem::*;
        match self {
            Fn(..) => "function signature",
            Const(..) => "associated constant",
            Type(..) => "associated type",
            Error(..) => "error",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemTrait {
    pub visibility: Option<PubToken>,
    pub trait_token: TraitToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub where_clause_opt: Option<WhereClause>,
    pub super_traits: Option<(ColonToken, Traits)>,
    pub trait_items: Braces<Vec<Annotated<ItemTraitItem>>>,
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
        Span::join(start, &end)
    }
}

impl Spanned for ItemTraitItem {
    fn span(&self) -> Span {
        match self {
            ItemTraitItem::Fn(fn_decl, semicolon) => match semicolon.as_ref().map(|x| x.span()) {
                Some(semicolon) => Span::join(fn_decl.span(), &semicolon),
                None => fn_decl.span(),
            },
            ItemTraitItem::Const(const_decl, semicolon) => {
                match semicolon.as_ref().map(|x| x.span()) {
                    Some(semicolon) => Span::join(const_decl.span(), &semicolon),
                    None => const_decl.span(),
                }
            }
            ItemTraitItem::Type(type_decl, semicolon) => {
                match semicolon.as_ref().map(|x| x.span()) {
                    Some(semicolon) => Span::join(type_decl.span(), &semicolon),
                    None => type_decl.span(),
                }
            }
            ItemTraitItem::Error(spans, _) => Span::join_all(spans.iter().cloned()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Traits {
    pub prefix: PathType,
    pub suffixes: Vec<(AddToken, PathType)>,
}

impl Traits {
    pub fn iter(&self) -> impl Iterator<Item = &PathType> {
        vec![&self.prefix]
            .into_iter()
            .chain(self.suffixes.iter().map(|(_add_token, path)| path))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PathType> {
        vec![&mut self.prefix]
            .into_iter()
            .chain(self.suffixes.iter_mut().map(|(_add_token, path)| path))
    }
}

impl Spanned for Traits {
    fn span(&self) -> Span {
        match self.suffixes.last() {
            Some((_add_token, path_type)) => Span::join(self.prefix.span(), &path_type.span()),
            None => self.prefix.span(),
        }
    }
}
