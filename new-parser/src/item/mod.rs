use crate::priv_prelude::*;

mod type_fields;
mod item_use;
mod item_struct;
mod item_enum;
mod item_fn;
mod item_trait;
mod item_abi;

pub use type_fields::*;
pub use item_use::*;
pub use item_struct::*;
pub use item_enum::*;
pub use item_fn::*;
pub use item_trait::*;
pub use item_abi::*;

pub enum Item {
    Use(ItemUse),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Function(ItemFn),
    Trait(ItemTrait),
    Abi(ItemAbi),
}

impl Spanned for Item {
    fn span(&self) -> Span {
        match self {
            Item::Use(item_use) => item_use.span(),
            Item::Struct(item_struct) => item_struct.span(),
            Item::Enum(item_enum) => item_enum.span(),
            Item::Function(item_fn) => item_fn.span(),
            Item::Trait(item_trait) => item_trait.span(),
            Item::Abi(item_abi) => item_abi.span(),
        }
    }
}

pub struct FnSignature {
    pub fn_token: FnToken,
    pub name: Ident,
    pub arguments: Parens<TypeFields>,
    pub return_type_opt: Option<(RightArrowToken, Ty)>,
}

impl Spanned for FnSignature {
    fn span(&self) -> Span {
        match &self.return_type_opt {
            Some((_return_type, ty)) => {
                Span::join(self.fn_token.span(), ty.span())
            },
            None => {
                Span::join(self.fn_token.span(), self.arguments.span())
            },
        }
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
    let item_abi = {
        item_abi()
        .map(Item::Abi)
    };

    item_use
    .or(item_struct)
    .or(item_enum)
    .or(item_fn)
    .or(item_trait)
    .or(item_abi)
}

pub fn fn_signature() -> impl Parser<Output = FnSignature> + Clone {
    fn_token()
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(parens(padded(type_fields())))
    .then_optional_whitespace()
    .then(
        right_arrow_token()
        .then_optional_whitespace()
        .then(ty())
        .then_optional_whitespace()
        .optional()
    )
    .map(|(((fn_token, name), arguments), return_type_opt): (_, Option<_>)| {
        FnSignature { fn_token, name, arguments, return_type_opt }
    })
}

