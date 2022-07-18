use crate::{
    fmt::{Format, FormattedCode, Formatter, FormatterError},
    utils::comments::{CommentSpan, CommentVisitor},
};
use sway_parse::{Item, ItemKind::*};

impl CommentVisitor for Item {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        match &self.value {
            Struct(item_struct) => item_struct.collect_spans(),
            Enum(item_enum) => item_enum.collect_spans(),
            Fn(item_fn) => item_fn.collect_spans(),
            Abi(item_abi) => item_abi.collect_spans(),
            _ => Vec::new(),
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
