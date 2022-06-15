use crate::{
    config::whitespace::NewlineStyle,
    utils::{
        indent_style::Shape, newline_style::apply_newline_style, program_type::insert_program_type,
    },
};
use std::{path::Path, sync::Arc};
use sway_core::BuildConfig;
use sway_parse::ItemKind;

use crate::utils::newline_style::apply_newline_style;
pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};

#[derive(Debug, Default)]
pub struct Formatter {
    pub shape: Shape,
    pub config: Config,
}

pub type FormattedCode = String;

pub trait Format {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode;
}

impl Formatter {
    pub fn from_dir(dir: &Path) -> Result<Self, ConfigError> {
        let config = match Config::from_dir(dir) {
            Ok(config) => config,
            Err(ConfigError::NotFound) => Config::default(),
            Err(e) => return Err(e),
        };
        let shape = Shape::default();
        Ok(Self { config, shape })
    }
    pub fn format(
        &mut self,
        src: Arc<str>,
        build_config: Option<&BuildConfig>,
    ) -> Result<FormattedCode, FormatterError> {
        let path = build_config.map(|build_config| build_config.canonical_root_module());
        let module = sway_parse::parse_file(src, path)?;
        // Get parsed items
        let items = module.items;
        // Get the program type (script, predicate, contract or library)
        let program_type = module.kind;

        // Formatted code will be pushed here with raw newline stlye.
        // Which means newlines are not converted into system-specific versions by apply_newline_style
        let mut formatted_raw_newline = String::new();

        // Insert program type to the formatted code.
        insert_program_type(&mut formatted_raw_newline, program_type);
        // Insert parsed & formatted items into the formatted code.
        formatted_raw_newline += &items
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
            .join("\n");
        let mut formatted_code = String::from(&formatted_raw_newline);
        apply_newline_style(
            // The user's setting for `NewlineStyle`
            self.config.whitespace.newline_style,
            &mut formatted_code,
            &formatted_raw_newline,
        );
        Ok(formatted_code)
    }
}
