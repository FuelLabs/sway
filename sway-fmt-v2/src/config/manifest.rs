use anyhow::{anyhow, Result};
use forc_util::println_yellow_err;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::config::{
    comments::Comments, expr::Expressions, heuristics::Heuristics, imports::Imports, items::Items,
    lists::Lists, literals::Literals, ordering::Ordering, user_def::Structures,
    whitespace::Whitespace,
};
pub use crate::error::FormatterError;

/// A direct mapping to a `swayfmt.toml`.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub struct FormatConfig {
    pub whitespace: Whitespace,
    pub imports: Imports,
    pub ordering: Ordering,
    pub items: Items,
    pub lists: Lists,
    pub literals: Literals,
    pub expressions: Expressions,
    pub heuristics: Heuristics,
    pub structures: Structures,
    pub comments: Comments,
}

impl FormatConfig {
    /// The default setting of `swayfmt`'s `FormatConfig`.
    ///
    pub fn default() -> Self {
        Self {
            whitespace: Whitespace::default(),
            imports: Imports::default(),
            ordering: Ordering::default(),
            items: Items::default(),
            lists: Lists::default(),
            literals: Literals::default(),
            expressions: Expressions::default(),
            heuristics: Heuristics::default(),
            structures: Structures::default(),
            comments: Comments::default(),
        }
    }

    /// Given an optional path to a `swayfmt.toml`, read it and construct a `FormatConfig`.
    /// If settings are omitted, those fields will be set to default. If `None` is provided,
    /// the default config will be used.
    ///
    /// At present, this will only return a warning if it catches unusable fields.
    /// Upon completion, this should give errors/warnings for incorrect input fields.
    ///
    pub fn from_file_or_default(config_path: Option<&Path>) -> Result<Self> {
        match config_path {
            Some(path) => {
                Self::default();
                let config_str = std::fs::read_to_string(path)
                    .map_err(|e| anyhow!("failed to read config at {:?}: {}", path, e))?;
                let toml_de = &mut toml::de::Deserializer::new(&config_str);
                let user_settings: Self = serde_ignored::deserialize(toml_de, |field| {
                    let warning = format!("  WARNING! found unusable configuration: {}", field);
                    println_yellow_err(&warning);
                })
                .map_err(|e| anyhow!("failed to parse config: {}.", e))?;

                Ok(Self::apply_user_settings(user_settings))
            }
            None => Ok(Self::default()),
        }
    }

    /// Check the user's settings, and replace the values of the default formatter if they exist.
    ///
    pub fn apply_user_settings(user_settings: FormatConfig) -> Self {
        Self {
            whitespace: user_settings.whitespace,
            imports: user_settings.imports,
            ordering: user_settings.ordering,
            items: user_settings.items,
            lists: user_settings.lists,
            literals: user_settings.literals,
            expressions: user_settings.expressions,
            heuristics: user_settings.heuristics,
            structures: user_settings.structures,
            comments: user_settings.comments,
        }
    }
}
