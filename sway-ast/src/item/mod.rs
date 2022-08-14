use crate::priv_prelude::*;

pub mod item_abi;
pub mod item_const;
pub mod item_control_flow;
pub mod item_enum;
pub mod item_fn;
pub mod item_impl;
pub mod item_storage;
pub mod item_struct;
pub mod item_trait;
pub mod item_use;

pub type Item = Annotated<ItemKind>;

impl Spanned for Item {
    fn span(&self) -> Span {
        match self.attribute_list.first() {
            Some(attr0) => Span::join(attr0.span(), self.value.span()),
            None => self.value.span(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ItemKind {
    Use(ItemUse),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Fn(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Abi(ItemAbi),
    Const(ItemConst),
    Storage(ItemStorage),
    Break(ItemBreak),
    Continue(ItemContinue),
}

impl Spanned for ItemKind {
    fn span(&self) -> Span {
        match self {
            ItemKind::Use(item_use) => item_use.span(),
            ItemKind::Struct(item_struct) => item_struct.span(),
            ItemKind::Enum(item_enum) => item_enum.span(),
            ItemKind::Fn(item_fn) => item_fn.span(),
            ItemKind::Trait(item_trait) => item_trait.span(),
            ItemKind::Impl(item_impl) => item_impl.span(),
            ItemKind::Abi(item_abi) => item_abi.span(),
            ItemKind::Const(item_const) => item_const.span(),
            ItemKind::Storage(item_storage) => item_storage.span(),
            ItemKind::Break(item_break) => item_break.span(),
            ItemKind::Continue(item_continue) => item_continue.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl Spanned for TypeField {
    fn span(&self) -> Span {
        Span::join(self.name.span(), self.ty.span())
    }
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Static(Punctuated<FnArg, CommaToken>),
    NonStatic {
        self_token: SelfToken,
        ref_self: Option<RefToken>,
        mutable_self: Option<MutToken>,
        args_opt: Option<(CommaToken, Punctuated<FnArg, CommaToken>)>,
    },
}

#[derive(Clone, Debug)]
pub struct FnArg {
    pub pattern: Pattern,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl Spanned for FnArg {
    fn span(&self) -> Span {
        Span::join(self.pattern.span(), self.ty.span())
    }
}

#[derive(Clone, Debug)]
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
        Span::join(start, end)
    }
}
