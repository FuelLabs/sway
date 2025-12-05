pub use crate::error::FormatterError;
use crate::{
    config::{
        comments::Comments, expr::Expressions, heuristics::Heuristics, imports::Imports,
        items::Items, literals::Literals, ordering::Ordering, user_def::Structures, user_opts::*,
        whitespace::Whitespace,
    },
    constants::SWAY_FORMAT_FILE_NAME,
    error::ConfigError,
};
use forc_diagnostic::println_yellow_err;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sway_utils::find_parent_dir_with_file;

/// A finalized `swayfmt` config.
#[derive(Debug, Default, Clone)]
pub struct Config {
    pub whitespace: Whitespace,
    pub imports: Imports,
    pub ordering: Ordering,
    pub items: Items,
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
    pub literals: Option<LiteralsOptions>,
    pub expressions: Option<ExpressionsOptions>,
    pub heuristics: Option<HeuristicsOptions>,
    pub structures: Option<StructuresOptions>,
    pub comments: Option<CommentsOptions>,
}

impl Config {
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
    /// Given a directory to a forc project containing a `swayfmt.toml`, read and
    /// construct a `Config` from the project's `swayfmt.toml` configuration file.
    ///
    /// This is a combination of `ConfigOptions::from_dir` and `Config::from_opts`,
    /// and takes care of constructing a finalized config.
    pub fn from_dir(config_path: &Path) -> Result<Self, ConfigError> {
        let config_opts = ConfigOptions::from_dir(config_path)?;
        Ok(Self::from_opts(config_opts))
    }
}

impl ConfigOptions {
    /// Given a path to a `swayfmt.toml`, read and construct the `ConfigOptions`.
    pub fn from_file(config_path: PathBuf) -> Result<Self, ConfigError> {
        let config_str =
            std::fs::read_to_string(&config_path).map_err(|e| ConfigError::ReadConfig {
                path: config_path,
                err: e,
            })?;
        let toml_de = toml::de::Deserializer::new(&config_str);
        let config_opts: Self = serde_ignored::deserialize(toml_de, |field| {
            let warning = format!("  WARNING! found unusable configuration: {field}");
            println_yellow_err(&warning);
        })
        .map_err(|e| ConfigError::Deserialize { err: (e) })?;
        Ok(config_opts)
    }
    /// Given a directory to a forc project containing a `swayfmt.toml`, read the config.
    ///
    /// This is short for `ConfigOptions::from_file`, but takes care of constructing the path to the
    /// file.
    pub fn from_dir(dir: &Path) -> Result<Self, ConfigError> {
        let config_dir =
            find_parent_dir_with_file(dir, SWAY_FORMAT_FILE_NAME).ok_or(ConfigError::NotFound)?;
        let file_path = config_dir.join(SWAY_FORMAT_FILE_NAME);
        Self::from_file(file_path)
    }
}
