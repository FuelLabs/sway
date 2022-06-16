use crate::utils::indent_style::Shape;
use std::{path::Path, sync::Arc};
use sway_core::BuildConfig;
use sway_parse::ItemKind;

pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};
use crate::{config::whitespace::NewlineStyle, utils::newline_style::apply_newline_style};

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
        let items = sway_parse::parse_file(src, path)?.items;
        let formatted_raw_newline = items
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
            NewlineStyle::Auto,
            &mut formatted_code,
            &formatted_raw_newline,
        );
        Ok(formatted_code)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::indent_style::Shape;
    use std::sync::Arc;

    use super::{Config, Formatter};

    fn get_formatter(config: Config, shape: Shape) -> Formatter {
        Formatter { config, shape }
    }

    #[test]
    fn test_enum_without_variant_alignment() {
        let sway_code_to_format = r#"contract;

enum Color {
    Blue: (), Green: (),
            Red: (),
    Silver: (),
                    Grey: (), }
        "#;

        // Until #1995 is addressed we will not have contract; in the output
        let correct_sway_code = r#"enum Color {
 Blue : (),
 Green : (),
 Red : (),
 Silver : (),
 Grey : (),
}"#;
        let mut formatter = get_formatter(Config::default(), Shape::default());
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert!(correct_sway_code == formatted_sway_code)
    }
    #[test]
    fn test_enum_with_variant_alignment() {
        let sway_code_to_format = r#"contract;

enum Color {
    Blue: (), Green: (),
            Red: (),
    Silver: (),
                    Grey: (), }
        "#;

        // Until #1995 is addressed we will not have contract; in the output
        let correct_sway_code = r#"enum Color {
 Blue   : (),
 Green  : (),
 Red    : (),
 Silver : (),
 Grey   : (),
}"#;

        // Creating a config with enum_variant_align_threshold that exceeds longest variant length
        let mut config = Config::default();
        config.structures.enum_variant_align_threshold = 20;

        let mut formatter = get_formatter(config, Shape::default());
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert!(correct_sway_code == formatted_sway_code)
    }
}
