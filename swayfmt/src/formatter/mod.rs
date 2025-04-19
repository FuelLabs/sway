use self::shape::Shape;
use crate::comments::{write_comments, CommentsContext};
use crate::parse::parse_file;
use crate::utils::map::comments::CommentMap;
use crate::utils::map::{newline::handle_newlines, newline_style::apply_newline_style};
pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};
use std::{borrow::Cow, fmt::Write, path::Path, sync::Arc};
use sway_ast::attribute::Annotated;
use sway_ast::Module;
use sway_types::span::Source;
use sway_types::{SourceEngine, Spanned};

pub(crate) mod shape;

#[derive(Debug, Default, Clone)]
pub struct Formatter {
    pub source_engine: Arc<SourceEngine>,
    pub shape: Shape,
    pub config: Config,
    pub comments_context: CommentsContext,
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

    /// Adds a block to the indentation level of the current [`Shape`].
    pub fn indent(&mut self) {
        self.shape.block_indent(&self.config);
    }

    /// Removes a block from the indentation level of the current [`Shape`].
    pub fn unindent(&mut self) {
        self.shape.block_unindent(&self.config);
    }

    /// Returns the [`Shape`]'s indentation blocks as a `Cow<'static, str>`.
    pub fn indent_to_str(&self) -> Result<Cow<'static, str>, FormatterError> {
        self.shape.indent.to_string(&self.config)
    }

    /// Checks the current level of indentation before writing the indent str into the buffer.
    pub fn write_indent_into_buffer(
        &self,
        formatted_code: &mut FormattedCode,
    ) -> Result<(), FormatterError> {
        let indent_str = self.indent_to_str()?;
        if !formatted_code.ends_with(indent_str.as_ref()) {
            write!(formatted_code, "{indent_str}")?
        }
        Ok(())
    }

    /// Collect a mapping of Span -> Comment from unformatted input.
    pub fn with_comments_context(&mut self, src: &str) -> Result<&mut Self, FormatterError> {
        let comments_context =
            CommentsContext::new(CommentMap::from_src(src.into())?, src.to_string());
        self.comments_context = comments_context;
        Ok(self)
    }

    pub fn format(&mut self, src: Source) -> Result<FormattedCode, FormatterError> {
        let annotated_module = parse_file(src)?;
        self.format_module(&annotated_module)
    }

    pub fn format_module(
        &mut self,
        annotated_module: &Annotated<Module>,
    ) -> Result<FormattedCode, FormatterError> {
        // apply the width heuristics settings from the `Config`
        self.shape.apply_width_heuristics(
            self.config
                .heuristics
                .heuristics_pref
                .to_width_heuristics(self.config.whitespace.max_width),
        );

        // Get the original trimmed source code.
        let module_kind_span = annotated_module.value.kind.span();
        let src = module_kind_span.src().text.trim();

        // Formatted code will be pushed here with raw newline style.
        // Which means newlines are not converted into system-specific versions until `apply_newline_style()`.
        // Use the length of src as a hint of the memory size needed for `raw_formatted_code`,
        // which will reduce the number of reallocations
        let mut raw_formatted_code = String::with_capacity(src.len());

        self.with_comments_context(src)?;

        annotated_module.format(&mut raw_formatted_code, self)?;

        let mut formatted_code = String::from(&raw_formatted_code);

        // Write post-module comments
        write_comments(
            &mut formatted_code,
            annotated_module.value.span().end()..src.len() + 1,
            self,
        )?;

        // Add newline sequences
        handle_newlines(
            Arc::from(src),
            &annotated_module.value,
            formatted_code.as_str().into(),
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
