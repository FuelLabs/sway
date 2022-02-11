use crate::priv_prelude::*;

mod type_fields;
mod item_use;
mod item_struct;
mod item_enum;
mod item_fn;
mod item_trait;
mod item_impl;
mod item_abi;
mod item_const;
mod item_storage;

pub use type_fields::*;
pub use item_use::*;
pub use item_struct::*;
pub use item_enum::*;
pub use item_fn::*;
pub use item_trait::*;
pub use item_impl::*;
pub use item_abi::*;
pub use item_const::*;
pub use item_storage::*;

#[derive(Clone, Debug)]
pub enum Item {
    Use(ItemUse),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Function(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Abi(ItemAbi),
    Const(ItemConst),
    Storage(ItemStorage),
}

impl Spanned for Item {
    fn span(&self) -> Span {
        match self {
            Item::Use(item_use) => item_use.span(),
            Item::Struct(item_struct) => item_struct.span(),
            Item::Enum(item_enum) => item_enum.span(),
            Item::Function(item_fn) => item_fn.span(),
            Item::Trait(item_trait) => item_trait.span(),
            Item::Impl(item_impl) => item_impl.span(),
            Item::Abi(item_abi) => item_abi.span(),
            Item::Const(item_const) => item_const.span(),
            Item::Storage(item_storage) => item_storage.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Static(TypeFields),
    NonStatic {
        self_token: SelfToken,
        args_opt: Option<(CommaToken, TypeFields)>,
    },
}

impl Spanned for FnArgs {
    fn span(&self) -> Span {
        match self {
            FnArgs::Static(type_fields) => type_fields.span(),
            FnArgs::NonStatic { self_token, args_opt } => {
                match args_opt {
                    Some((_, type_fields)) => Span::join(self_token.span(), type_fields.span()),
                    None => self_token.span(),
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FnSignature {
    pub visibility: Option<PubToken>,
    pub impure: Option<ImpureToken>,
    pub fn_token: FnToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub arguments: Parens<FnArgs>,
    pub return_type_opt: Option<(RightArrowToken, Ty)>,
}

impl Spanned for FnSignature {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => match &self.impure {
                Some(impure_token) => impure_token.span(),
                None => self.fn_token.span(),
            },
        };
        let end = match &self.return_type_opt {
            Some((_right_arrow_token, ty)) => ty.span(),
            None => self.arguments.span(),
        };
        Span::join(start, end)
    }
}

pub fn item() -> impl Parser<Output = Item> + Clone {
    let item_use = {
        item_use()
        .map(Item::Use)
    };
    let item_struct = {
        item_struct()
        .map(Item::Struct)
    };
    let item_enum = {
        item_enum()
        .map(Item::Enum)
    };
    let item_fn = {
        item_fn()
        .map(Item::Function)
    };
    let item_trait = {
        item_trait()
        .map(Item::Trait)
    };
    let item_impl = {
        item_impl()
        .map(Item::Impl)
    };
    let item_abi = {
        item_abi()
        .map(Item::Abi)
    };
    let item_const = {
        item_const()
        .map(Item::Const)
    };
    let item_storage = {
        item_storage()
        .map(Item::Storage)
    };

    or! {
        item_use,
        item_struct,
        item_enum,
        item_fn,
        item_trait,
        item_impl,
        item_abi,
        item_const,
        item_storage,
    }
    .try_map_with_span(|item_opt: Option<Item>, span| {
        item_opt.ok_or_else(|| ParseError::ExpectedItem { span })
    })
}

pub fn fn_args() -> impl Parser<Output = FnArgs> + Clone {
    let args_static = {
        type_fields()
        .map(|type_fields| FnArgs::Static(type_fields))
    };
    let args_non_static = {
        self_token()
        .then_optional_whitespace()
        .then(
            comma_token()
            .then_optional_whitespace()
            .then(type_fields())
            .optional()
        )
        .map(|(self_token, args_opt)| FnArgs::NonStatic { self_token, args_opt })
    };

    args_non_static
    .or(args_static)
}

pub fn fn_signature() -> impl Parser<Output = FnSignature> + Clone {
    pub_token()
    .then_whitespace()
    .optional()
    .then(
        impure_token()
        .then_whitespace()
        .optional()
    )
    .then(fn_token())
    .then_whitespace()
    .commit()
    .then(ident())
    .then_optional_whitespace()
    .then(
        generic_params()
        .then_optional_whitespace()
        .optional()
    )
    .then(parens(fn_args()))
    .then_optional_whitespace()
    .then(
        right_arrow_token()
        .then_optional_whitespace()
        .then(ty())
        .then_optional_whitespace()
        .optional()
    )
    .map(|((((((visibility, impure), fn_token), name), generics), arguments), return_type_opt)| {
        FnSignature { visibility, impure, fn_token, name, generics, arguments, return_type_opt }
    })
}

