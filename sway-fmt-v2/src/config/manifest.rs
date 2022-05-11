use anyhow::{anyhow, Result};
use forc_util::{find_parent_dir_with_file, println_yellow_err};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use crate::error::FormatterError;
use crate::{
    config::{
        comments::Comments, expr::Expressions, heuristics::Heuristics, imports::Imports,
        items::Items, lists::Lists, literals::Literals, ordering::Ordering, user_def::Structures,
        user_opts::*, whitespace::Whitespace,
    },
    constants::SWAY_FORMAT_FILE_NAME,
};

/// A finalized `swayfmt` config.
#[derive(Debug)]
pub struct Config {
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

/// A direct mapping to an optional `swayfmt.toml`.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ConfigOptions {
    pub whitespace: Option<WhitespaceOptions>,
    pub imports: Option<ImportsOptions>,
    pub ordering: Option<OrderingOptions>,
    pub items: Option<ItemsOptions>,
    pub lists: Option<ListsOptions>,
    pub literals: Option<LiteralsOptions>,
    pub expressions: Option<ExpressionsOptions>,
    pub heuristics: Option<HeuristicsOptions>,
    pub structures: Option<StructuresOptions>,
    pub comments: Option<CommentsOptions>,
}

impl Config {
    /// The default setting of `swayfmt`'s `Config`.
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
    /// Construct the set of configuration to be used from the given set of options.
    ///
    pub fn from_opts(opts: ConfigOptions) -> Self {
        Self {
            whitespace: opts
                .whitespace
                .as_ref()
                .map(Whitespace::from_opts)
                .unwrap_or_default(),
            imports: opts
                .imports
                .as_ref()
                .map(Imports::from_opts)
                .unwrap_or_default(),
            ordering: opts
                .ordering
                .as_ref()
                .map(Ordering::from_opts)
                .unwrap_or_default(),
            items: opts
                .items
                .as_ref()
                .map(Items::from_opts)
                .unwrap_or_default(),
            lists: opts
                .lists
                .as_ref()
                .map(Lists::from_opts)
                .unwrap_or_default(),
            literals: opts
                .literals
                .as_ref()
                .map(Literals::from_opts)
                .unwrap_or_default(),
            expressions: opts
                .expressions
                .as_ref()
                .map(Expressions::from_opts)
                .unwrap_or_default(),
            heuristics: opts
                .heuristics
                .as_ref()
                .map(Heuristics::from_opts)
                .unwrap_or_default(),
            structures: opts
                .structures
                .as_ref()
                .map(Structures::from_opts)
                .unwrap_or_default(),
            comments: opts
                .comments
                .as_ref()
                .map(Comments::from_opts)
                .unwrap_or_default(),
        }
    }
    /// Given an optional path to a `swayfmt.toml`, read it and construct a `Config`.
    /// If settings are omitted, those fields will be set to default. If `None` is provided,
    /// the default config will be applied. If a `swayfmt.toml` exists but is empty, the default
    /// config will be applied.
    ///
    /// At present, this will only return a warning if it catches unusable fields.
    /// Upon completion, this should give errors/warnings for incorrect input fields as well.
    ///
    pub fn from_dir_or_default(config_path: Option<&Path>) -> Result<Self> {
        let config = ConfigOptions::from_dir_or_default(config_path)?;
        Ok(config)
    }
}

impl ConfigOptions {
    /// Given an optional path to a `swayfmt.toml`, read it and construct a `Config`.
    /// If settings are omitted, those fields will be set to default. If `None` is provided,
    /// the default config will be applied. If a `swayfmt.toml` exists but is empty, the default
    /// config will be applied.
    ///
    /// At present, this will only return a warning if it catches unusable fields.
    /// Upon completion, this should give errors/warnings for incorrect input fields as well.
    ///
    pub fn from_dir_or_default(config_path: Option<&Path>) -> Result<Config> {
        match config_path {
            Some(starter_path) => {
                if let Some(path) = find_parent_dir_with_file(starter_path, SWAY_FORMAT_FILE_NAME) {
                    let config_str = std::fs::read_to_string(&path)
                        .map_err(|e| anyhow!("failed to read config at {:?}: {}", path, e))?;
                    // save some time if the file is empty
                    if !config_str.is_empty() {
                        let toml_de = &mut toml::de::Deserializer::new(&config_str);
                        let user_settings: Self = serde_ignored::deserialize(toml_de, |field| {
                            let warning =
                                format!("  WARNING! found unusable configuration: {}", field);
                            println_yellow_err(&warning);
                        })
                        .map_err(|e| anyhow!("failed to parse config: {}.", e))?;

                        Ok(Config::from_opts(user_settings))
                    } else {
                        Ok(Config::default())
                    }
                } else {
                    Ok(Config::default())
                }
            }
            None => Ok(Config::default()),
        }
    }
}
