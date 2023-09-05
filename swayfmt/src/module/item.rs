use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use sway_ast::ItemKind::{self, *};

impl Format for ItemKind {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Submodule(item_mod) => item_mod.format(formatted_code, formatter),
            Use(item_use) => item_use.format(formatted_code, formatter),
            Struct(item_struct) => item_struct.format(formatted_code, formatter),
            Enum(item_enum) => item_enum.format(formatted_code, formatter),
            Fn(item_fn) => item_fn.format(formatted_code, formatter),
            Trait(item_trait) => item_trait.format(formatted_code, formatter),
            Impl(item_impl) => item_impl.format(formatted_code, formatter),
            Abi(item_abi) => item_abi.format(formatted_code, formatter),
            Const(item_const) => item_const.format(formatted_code, formatter),
            Storage(item_storage) => item_storage.format(formatted_code, formatter),
            Configurable(item_configurable) => item_configurable.format(formatted_code, formatter),
            TypeAlias(item_type_alias) => item_type_alias.format(formatted_code, formatter),
            Error(_, _) => Ok(()),
        }
    }
}

impl LeafSpans for ItemKind {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            Submodule(item_mod) => item_mod.leaf_spans(),
            Struct(item_struct) => item_struct.leaf_spans(),
            Enum(item_enum) => item_enum.leaf_spans(),
            Fn(item_fn) => item_fn.leaf_spans(),
            Abi(item_abi) => item_abi.leaf_spans(),
            Const(item_const) => item_const.leaf_spans(),
            Storage(item_storage) => item_storage.leaf_spans(),
            Trait(item_trait) => item_trait.leaf_spans(),
            Impl(item_impl) => item_impl.leaf_spans(),
            Use(item_use) => item_use.leaf_spans(),
            Configurable(item_configurable) => item_configurable.leaf_spans(),
            TypeAlias(item_type_alias) => item_type_alias.leaf_spans(),
            Error(spans, _) => {
                vec![sway_types::Span::join_all(spans.iter().cloned()).into()]
            }
        }
    }
}

pub trait ItemLenChars {
    fn len_chars(&self) -> Result<usize, FormatterError>;
}
