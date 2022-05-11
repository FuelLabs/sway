use std::sync::Arc;
use sway_core::BuildConfig;
use sway_parse::Item;

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
        let path = build_config.map(|build_config| build_config.path());
        let items = sway_parse::parse_file(src, path)?.items;
        Ok(items
            .into_iter()
            .map(|item| -> Result<String, FormatterError> {
                use Item::*;
                Ok(match item {
                    Use(use_stmt) => use_stmt.format(self),
                    // don't format if we don't have a formatter for this `Item`
                    otherwise => otherwise.span().as_str().to_string(),
                })
            })
            .collect::<Result<Vec<String>, _>>()?
            .join("\n"))
    }
}
