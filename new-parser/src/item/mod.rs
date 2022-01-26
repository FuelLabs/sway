use crate::priv_prelude::*;

mod type_fields;
mod item_use;
mod item_struct;
mod item_enum;
mod item_fn;

pub use type_fields::*;
pub use item_use::*;
pub use item_struct::*;
pub use item_enum::*;
pub use item_fn::*;

pub enum Item {
    Use(ItemUse),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Function(ItemFn),
}

impl Spanned for Item {
    fn span(&self) -> Span {
        match self {
            Item::Use(item_use) => item_use.span(),
            Item::Struct(item_struct) => item_struct.span(),
            Item::Enum(item_enum) => item_enum.span(),
            Item::Function(item_fn) => item_fn.span(),
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

    item_use
    .or(item_struct)
    .or(item_enum)
    .or(item_fn)
}

