use self::shape::Shape;
use crate::parse::parse_file;
use crate::utils::map::comments::CommentMap;
use crate::utils::map::{
    comments::handle_comments, newline::handle_newlines, newline_style::apply_newline_style,
};
pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};
use std::{fmt::Write, path::Path, sync::Arc};
use sway_core::BuildConfig;

pub(crate) mod shape;

#[derive(Debug, Default, Clone)]
pub struct Formatter {
    pub shape: Shape,
    pub config: Config,
    pub comment_map: CommentMap,
}

pub type FormattedCode = String;

pub trait Format {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

impl Formatter {
    pub fn from_dir(dir: &Path) -> Result<Self, ConfigError> {
        let config = match Config::from_dir(dir) {
            Ok(config) => config,
            Err(ConfigError::NotFound) => Config::default(),
            Err(e) => return Err(e),
        };

        Ok(Self {
            config,
            ..Default::default()
        })
    }
    pub fn format(
        &mut self,
        src: Arc<str>,
        build_config: Option<&BuildConfig>,
    ) -> Result<FormattedCode, FormatterError> {
        // apply the width heuristics settings from the `Config`
        self.shape.apply_width_heuristics(
            self.config
                .heuristics
                .heuristics_pref
                .to_width_heuristics(self.config.whitespace.max_width),
        );
        let src = src.trim();

        let path = build_config.map(|build_config| build_config.canonical_root_module());
        // Formatted code will be pushed here with raw newline stlye.
        // Which means newlines are not converted into system-specific versions until `apply_newline_style()`.
        // Use the length of src as a hint of the memory size needed for `raw_formatted_code`,
        // which will reduce the number of reallocations
        let mut raw_formatted_code = String::with_capacity(src.len());

        // Collect Span -> Comment mapping from unformatted input.
        self.comment_map = CommentMap::from_src(Arc::from(src))?;

        let module = parse_file(Arc::from(src), path.clone())?;
        module.format(&mut raw_formatted_code, self)?;

        let mut formatted_code = String::from(&raw_formatted_code);

        // Add comments
        handle_comments(
            Arc::from(src),
            &module,
            Arc::from(formatted_code.clone()),
            path.clone(),
            &mut formatted_code,
            &mut self.comment_map,
        )?;
        // Add newline sequences
        handle_newlines(
            Arc::from(src),
            &module,
            Arc::from(formatted_code.clone()),
            path,
            &mut formatted_code,
            self,
        )?;
        // Replace newlines with specified `NewlineStyle`
        apply_newline_style(
            self.config.whitespace.newline_style,
            &mut formatted_code,
            &raw_formatted_code,
        )?;
        if !formatted_code.ends_with('\n') {
            writeln!(formatted_code)?;
        }

        Ok(formatted_code)
    }
    pub(crate) fn with_shape<F, O>(&mut self, new_shape: Shape, f: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let prev_shape = self.shape;
        self.shape = new_shape;
        let output = f(self);
        self.shape = prev_shape;

        output // used to extract an output if needed
    }
}
