use std::sync::Arc;
use sway_parse::Item;
use thiserror::Error;

pub use crate::error::FormatterError;

#[derive(Debug, Clone, Copy)]
pub struct Formatter {
    pub align_fields: bool,
    pub tab_size: u32,
}
impl Formatter {
    pub fn format(&self, src: Arc<str>) -> Result<String, FormatterError> {
        let items = sway_parse::parse_file(file.clone(), todo!())?.items;
        Ok(items
            .into_iter()
            .map(|item| -> Result<String, FormatterError> {
                use Item::*;
                Ok(match item {
                    Use(use_stmt) => todo!("Format me!"),
                    // don't format if we don't have a formatter for this `Item`
                    otherwise => otherwise.span().as_str().to_string(),
                })
            })
            .collect::<Result<Vec<String>, _>>()?
            .join("\n"))
    }
}

impl Formatter {
    pub fn default() -> Self {
        Self {
            align_fields: true,
            tab_size: 4,
        }
    }
}
