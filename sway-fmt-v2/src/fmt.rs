use std::sync::Arc;
use sway_core::BuildConfig;
use sway_parse::ItemKind;

use crate::config::manifest::Config;
pub use crate::error::FormatterError;

#[derive(Debug)]
pub struct Formatter {
    pub config: Config,
}

pub type FormattedCode = String;

pub trait Format {
    fn format(&self, formatter: &Formatter) -> FormattedCode;
}

impl Formatter {
    pub fn format(
        &self,
        src: Arc<str>,
        build_config: Option<&BuildConfig>,
    ) -> Result<FormattedCode, FormatterError> {
        let path = build_config.map(|build_config| build_config.canonical_root_module());
        let items = sway_parse::parse_file(src, path)?.items;
        Ok(items
            .into_iter()
            .map(|item| -> Result<String, FormatterError> {
                use ItemKind::*;
                Ok(match item.value {
                    Use(item_use) => item_use.format(self),
                    Struct(item_struct) => item_struct.format(self),
                    Enum(item_enum) => item_enum.format(self),
                    Fn(item_fn) => item_fn.format(self),
                    Trait(item_trait) => item_trait.format(self),
                    Impl(item_impl) => item_impl.format(self),
                    Abi(item_abi) => item_abi.format(self),
                    Const(item_const) => item_const.format(self),
                    Storage(item_storage) => item_storage.format(self),
                })
            })
            .collect::<Result<Vec<String>, _>>()?
            .join("\n"))
    }
}
