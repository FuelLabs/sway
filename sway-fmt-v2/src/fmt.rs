use std::{path::Path, sync::Arc};
use sway_core::BuildConfig;
use sway_parse::ItemKind;

pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};
use crate::{
    config::whitespace::NewlineStyle,
    utils::{
        indent_style::{Indent, Shape},
        newline_style::apply_newline_style,
    },
};

#[derive(Debug, Default)]
pub struct Formatter {
    pub config: Config,
}

pub type FormattedCode = String;

pub trait Format {
    fn format(&self, formatter: &Formatter, shape: &mut Shape) -> FormattedCode;
}

impl Formatter {
    pub fn from_dir(dir: &Path) -> Result<Self, ConfigError> {
        let config = match Config::from_dir(dir) {
            Ok(config) => config,
            Err(ConfigError::NotFound) => Config::default(),
            Err(e) => return Err(e),
        };
        Ok(Self { config })
    }
    pub fn format(
        &self,
        src: Arc<str>,
        build_config: Option<&BuildConfig>,
    ) -> Result<FormattedCode, FormatterError> {
        let path = build_config.map(|build_config| build_config.canonical_root_module());
        // Current shape is non indented
        let mut shape = Shape::indented(Indent::empty(), self);
        let items = sway_parse::parse_file(src, path)?.items;
        let raw_formatted_code = items
            .into_iter()
            .map(|item| -> Result<String, FormatterError> {
                use ItemKind::*;
                Ok(match item.value {
                    Use(item_use) => item_use.format(self, &mut shape),
                    Struct(item_struct) => item_struct.format(self, &mut shape),
                    Enum(item_enum) => item_enum.format(self, &mut shape),
                    Fn(item_fn) => item_fn.format(self, &mut shape),
                    Trait(item_trait) => item_trait.format(self, &mut shape),
                    Impl(item_impl) => item_impl.format(self, &mut shape),
                    Abi(item_abi) => item_abi.format(self, &mut shape),
                    Const(item_const) => item_const.format(self, &mut shape),
                    Storage(item_storage) => item_storage.format(self, &mut shape),
                })
            })
            .collect::<Result<Vec<String>, _>>()?
            .join("\n");
        let mut formatted_code = String::from(&raw_formatted_code);
        apply_newline_style(NewlineStyle::Auto, &mut formatted_code, &raw_formatted_code);
        Ok(formatted_code)
    }
}
