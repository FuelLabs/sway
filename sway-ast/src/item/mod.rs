use sway_error::handler::ErrorEmitted;

use crate::priv_prelude::*;

pub mod item_abi;
pub mod item_configurable;
pub mod item_const;
pub mod item_enum;
pub mod item_fn;
pub mod item_impl;
pub mod item_storage;
pub mod item_struct;
pub mod item_trait;
pub mod item_type_alias;
pub mod item_use;

pub type Item = Annotated<ItemKind>;

impl Spanned for Item {
    fn span(&self) -> Span {
        match self.attributes.first() {
            Some(attr0) => Span::join(attr0.span(), &self.value.span()),
            None => self.value.span(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize)]
pub enum ItemKind {
    Submodule(Submodule),
    Use(ItemUse),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Fn(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Abi(ItemAbi),
    Const(ItemConst),
    Storage(ItemStorage),
    Configurable(ItemConfigurable),
    TypeAlias(ItemTypeAlias),
    // to handle parser recovery: Error represents an incomplete item
    Error(Box<[Span]>, #[serde(skip_serializing)] ErrorEmitted),
}

impl ItemKind {
    /// [ItemKind]'s friendly name string used for various reportings.
    ///
    /// Note that all friendly names are lowercase.
    /// This is also the case for names containing acronyms like ABI.
    /// For contexts in which acronyms need to be uppercase, like
    /// e.g., error reporting, use `friendly_name_with_acronym` instead.
    pub fn friendly_name(&self) -> &'static str {
        use ItemKind::*;
        match self {
            Submodule(_) => "submodule (`mod`)",
            Use(_) => "import (`use`)",
            Struct(_) => "struct declaration",
            Enum(_) => "enum declaration",
            Fn(_) => "function declaration",
            Trait(_) => "trait declaration",
            Impl(item_impl) => match item_impl.trait_opt {
                Some(_) => "ABI or trait implementation",
                None => "inherent implementation",
            },
            Abi(_) => "abi declaration",
            Const(_) => "constant declaration",
            Storage(_) => "contract storage declaration",
            Configurable(_) => "configurable declaration",
            TypeAlias(_) => "type alias declaration",
            Error(..) => "error",
        }
    }

    pub fn friendly_name_with_acronym(&self) -> &'static str {
        match self.friendly_name() {
            "abi declaration" => "ABI declaration",
            friendly_name => friendly_name,
        }
    }
}

impl Spanned for ItemKind {
    fn span(&self) -> Span {
        match self {
            ItemKind::Submodule(item_mod) => item_mod.span(),
            ItemKind::Use(item_use) => item_use.span(),
            ItemKind::Struct(item_struct) => item_struct.span(),
            ItemKind::Enum(item_enum) => item_enum.span(),
            ItemKind::Fn(item_fn) => item_fn.span(),
            ItemKind::Trait(item_trait) => item_trait.span(),
            ItemKind::Impl(item_impl) => item_impl.span(),
            ItemKind::Abi(item_abi) => item_abi.span(),
            ItemKind::Const(item_const) => item_const.span(),
            ItemKind::Storage(item_storage) => item_storage.span(),
            ItemKind::Configurable(item_configurable) => item_configurable.span(),
            ItemKind::TypeAlias(item_type_alias) => item_type_alias.span(),
            ItemKind::Error(spans, _) => Span::join_all(spans.iter().cloned()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TypeField {
    pub visibility: Option<PubToken>,
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl Spanned for TypeField {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.name.span(),
        };
        let end = self.ty.span();
        Span::join(start, &end)
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum FnArgs {
    Static(Punctuated<FnArg, CommaToken>),
    NonStatic {
        self_token: SelfToken,
        ref_self: Option<RefToken>,
        mutable_self: Option<MutToken>,
        args_opt: Option<(CommaToken, Punctuated<FnArg, CommaToken>)>,
    },
}

impl FnArgs {
    /// Returns all the [FnArg]s, from the function signature defined by `self`.
    ///
    /// If the `self` is [FnArgs::NonStatic], the first `self` argument is not
    /// returned, because it is not an [FnArg].
    pub fn args(&self) -> Vec<&FnArg> {
        match self {
            Self::Static(punctuated) => punctuated.iter().collect(),
            Self::NonStatic { args_opt, .. } => args_opt
                .as_ref()
                .map_or(vec![], |(_comma_token, punctuated)| {
                    punctuated.iter().collect()
                }),
        }
    }

    /// Returns all the [FnArg]s, from the function signature defined by `self`.
    ///
    /// If the `self` is [FnArgs::NonStatic], the first `self` argument is not
    /// returned, because it is not an [FnArg].
    pub fn args_mut(&mut self) -> Vec<&mut FnArg> {
        match self {
            Self::Static(punctuated) => punctuated.iter_mut().collect(),
            Self::NonStatic { args_opt, .. } => args_opt
                .as_mut()
                .map_or(vec![], |(_comma_token, punctuated)| {
                    punctuated.iter_mut().collect()
                }),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FnArg {
    pub pattern: Pattern,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl Spanned for FnArg {
    fn span(&self) -> Span {
        Span::join(self.pattern.span(), &self.ty.span())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FnSignature {
    pub visibility: Option<PubToken>,
    pub fn_token: FnToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub arguments: Parens<FnArgs>,
    pub return_type_opt: Option<(RightArrowToken, Ty)>,
    pub where_clause_opt: Option<WhereClause>,
}

impl Spanned for FnSignature {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.fn_token.span(),
        };
        let end = match &self.where_clause_opt {
            Some(where_clause) => where_clause.span(),
            None => match &self.return_type_opt {
                Some((_right_arrow, ty)) => ty.span(),
                None => self.arguments.span(),
            },
        };
        Span::join(start, &end)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TraitType {
    pub name: Ident,
    pub type_token: TypeToken,
    pub eq_token_opt: Option<EqToken>,
    pub ty_opt: Option<Ty>,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for TraitType {
    fn span(&self) -> Span {
        let start = self.type_token.span();
        let end = match &self.ty_opt {
            Some(ty_opt) => ty_opt.span(),
            None => self.name.span(),
        };
        Span::join(start, &end)
    }
}
