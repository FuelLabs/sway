use sway_parse::{Item, ItemKind::*};

use crate::fmt::{Format, FormattedCode, Formatter};

impl Format for Item {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        match &self.value {
            Use(item_use) => item_use.format(formatter),
            Struct(item_struct) => item_struct.format(formatter),
            Enum(item_enum) => item_enum.format(formatter),
            Fn(item_fn) => item_fn.format(formatter),
            Trait(item_trait) => item_trait.format(formatter),
            Impl(item_impl) => item_impl.format(formatter),
            Abi(item_abi) => item_abi.format(formatter),
            Const(item_const) => item_const.format(formatter),
            Storage(item_storage) => item_storage.format(formatter),
        }
    }
}
