use crate::{
    fmt::*,
    utils::comments::{ByteSpan, LeafSpans},
};
use sway_ast::{Item, ItemKind::*};

impl LeafSpans for Item {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match &self.value {
            Struct(item_struct) => item_struct.leaf_spans(),
            Enum(item_enum) => item_enum.leaf_spans(),
            Fn(item_fn) => item_fn.leaf_spans(),
            Abi(item_abi) => item_abi.leaf_spans(),
            Const(item_const) => item_const.leaf_spans(),
            Storage(item_storage) => item_storage.leaf_spans(),
            Trait(item_trait) => item_trait.leaf_spans(),
            Impl(item_impl) => item_impl.leaf_spans(),
            Use(item_use) => item_use.leaf_spans(),
            Break(_) => todo!(),
            Continue(_) => todo!(),
        }
    }
}

impl Format for Item {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match &self.value {
            Use(item_use) => item_use.format(formatted_code, formatter),
            Struct(item_struct) => item_struct.format(formatted_code, formatter),
            Enum(item_enum) => item_enum.format(formatted_code, formatter),
            Fn(item_fn) => item_fn.format(formatted_code, formatter),
            Trait(item_trait) => item_trait.format(formatted_code, formatter),
            Impl(item_impl) => item_impl.format(formatted_code, formatter),
            Abi(item_abi) => item_abi.format(formatted_code, formatter),
            Const(item_const) => item_const.format(formatted_code, formatter),
            Storage(item_storage) => item_storage.format(formatted_code, formatter),
            Break(_item_break) => todo!(),
            Continue(_item_continue) => todo!(),
        }
    }
}
pub trait ItemLenChars {
    fn len_chars(&self) -> Result<usize, FormatterError>;
}
