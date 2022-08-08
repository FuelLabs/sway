//! Associated functions and tests for handling of indentation.
use crate::{
    config::{heuristics::WidthHeuristics, manifest::Config},
    constants::{HARD_TAB, INDENT_BUFFER, INDENT_BUFFER_LEN},
    FormatterError,
};
use std::{
    borrow::Cow,
    fmt::Write,
    ops::{Add, Sub},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct Indent {
    /// Width of the block indent, in characters.
    /// Must be a multiple of `tab_spaces`.
    pub(crate) block_indent: usize,
}

impl Indent {
    /// Constructs a new instance of `Indent` with a given size.
    fn new(block_indent: usize) -> Self {
        Self { block_indent }
    }
    /// Constructs an empty instance of `Indent`.
    fn empty() -> Self {
        Self::new(0)
    }
    /// Adds a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    fn block_indent(&mut self, config: &Config) {
        self.block_indent += config.whitespace.tab_spaces
    }
    /// Removes a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    /// If the current level of indentation would be negative, leave it as is.
    fn block_unindent(&mut self, config: &Config) {
        let tab_spaces = config.whitespace.tab_spaces;
        if self.block_indent < tab_spaces {
        } else {
            self.block_indent -= tab_spaces
        }
    }
    /// Current indent size.
    pub(crate) fn width(&self) -> usize {
        self.block_indent
    }
    // Checks for either `hard_tabs` or `tab_spaces` and creates a
    // buffer of whitespace from the current level of indentation.
    // This also takes an offset which determines whether to add a new line.
    fn to_string_inner(
        self,
        config: &Config,
        offset: usize,
    ) -> Result<Cow<'static, str>, FormatterError> {
        let (num_tabs, num_spaces) = if config.whitespace.hard_tabs {
            (self.block_indent / config.whitespace.tab_spaces, 0)
        } else {
            (0, self.width())
        };
        let num_chars = num_tabs + num_spaces;
        if num_tabs == 0 && num_chars + offset <= INDENT_BUFFER_LEN {
            Ok(Cow::from(&INDENT_BUFFER[offset..=num_chars]))
        } else {
            let mut indent = String::with_capacity(num_chars + if offset == 0 { 1 } else { 0 });
            if offset == 0 {
                writeln!(indent)?;
            }
            for _ in 0..num_tabs {
                write!(indent, "{}", HARD_TAB)?;
            }
            for _ in 0..num_spaces {
                write!(indent, " ")?;
            }
            Ok(Cow::from(indent))
        }
    }
    /// A wrapper for `Indent::to_string_inner()` that does not add a new line.
    pub(crate) fn to_string(self, config: &Config) -> Result<Cow<'static, str>, FormatterError> {
        self.to_string_inner(config, 1)
    }
    /// A wrapper for `Indent::to_string_inner()` that also adds a new line.
    pub(crate) fn to_string_with_newline(
        self,
        config: &Config,
    ) -> Result<Cow<'static, str>, FormatterError> {
        self.to_string_inner(config, 0)
    }
}

impl Add for Indent {
    type Output = Indent;

    fn add(self, rhs: Indent) -> Indent {
        Indent {
            block_indent: self.block_indent + rhs.block_indent,
        }
    }
}

impl Sub for Indent {
    type Output = Indent;

    fn sub(self, rhs: Indent) -> Indent {
        Indent::new(self.block_indent - rhs.block_indent)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct CodeLine {
    pub(crate) line_style: LineStyle,
    pub(crate) expr_kind: ExprKind,
}

impl CodeLine {
    /// Update `CodeLine::line_style` with a given LineStyle.
    pub(crate) fn update_line_style(&mut self, line_style: LineStyle) {
        self.line_style = line_style
    }
    /// Update `CodeLine::expr_kind` with a given ExprKind.
    pub(crate) fn update_expr_kind(&mut self, expr_kind: ExprKind) {
        self.expr_kind = expr_kind
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum LineStyle {
    Normal,
    Inline,
    Multiline,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self::Normal
    }
}

/// The type of expression to determine which part of `Config::heuristics` to use.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExprKind {
    Variable,
    Function,
    Struct,
    Collection,
    MethodChain,
    Conditional,
    Undetermined,
}

impl Default for ExprKind {
    fn default() -> Self {
        Self::Undetermined
    }
}

/// The current shape of the formatter.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct Shape {
    /// The current number of characters in the given code line.
    pub(crate) width: usize,
    /// The current indentation of code.
    pub(crate) indent: Indent,
    /// Used in determining which heuristics settings to use from the config
    /// and whether a code line is normal, inline or multiline.
    pub(crate) code_line: CodeLine,
    /// The definitive width settings from the `Config`.
    pub(crate) width_heuristics: WidthHeuristics,
    /// Used in determining `SameLineWhere` formatting.
    /// Default is false.
    pub(crate) has_where_clause: bool,
}

impl Shape {
    // `indent` is the indentation of the first line. The next lines
    // should begin with at least `indent` spaces (except backwards
    // indentation). The first line should not begin with indentation.
    // `width` is the maximum number of characters on the last line
    // (excluding `indent`). The width of other lines is not limited by
    // `width`.
    //
    // Note that in reality, we sometimes use width for lines other than the
    // last (i.e., we are conservative).
    // .......*-------*
    //        |       |
    //        |     *-*
    //        *-----|
    // |<------------>|  max width
    // |<---->|          indent
    //        |<--->|    width
    //
    /// A wrapper for `to_width_heuristics()` that also checks if the settings are default.
    pub(crate) fn apply_width_heuristics(&mut self, width_heuristics: WidthHeuristics) {
        if width_heuristics == WidthHeuristics::default() {
        } else {
            self.width_heuristics = width_heuristics
        }
    }
    /// A wrapper for `Indent::block_indent()`.
    ///
    /// Adds a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    pub(crate) fn block_indent(&mut self, config: &Config) {
        self.indent.block_indent(config)
    }
    /// A wrapper for `Indent::block_unindent()`.
    ///
    /// Removes a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    /// If the current level of indentation would be negative, leave it as is.
    pub(crate) fn block_unindent(&mut self, config: &Config) {
        self.indent.block_unindent(config)
    }
    /// Updates `Shape::width` to the current width of the `Item`.
    pub(crate) fn update_width(&mut self, len_chars: usize) {
        self.width = len_chars
    }
    pub(crate) fn add_width(&mut self, len_chars: usize) {
        self.width += len_chars
    }
    pub(crate) fn sub_width(&mut self, len_chars: usize) {
        self.width -= len_chars
    }
    pub(crate) fn reset_width(&mut self) {
        self.update_width(0)
    }
    /// Checks the config, and if `small_structure_single_line` is enabled,
    /// determines whether the `Shape::width` is greater than the `structure_lit_width`
    /// threshold. If it isn't, the `Shape::line_style` is updated to `Inline`.
    pub(crate) fn get_line_style(
        &mut self,
        field_width: Option<usize>,
        body_width: usize,
        config: &Config,
    ) {
        // we can maybe do this at the beginning and store it on shape since it won't change during formatting
        // let width_heuristics = config
        //     .heuristics
        //     .heuristics_pref
        //     .to_width_heuristics(config.whitespace.max_width);
        match self.code_line.expr_kind {
            ExprKind::Struct => {
                // Get the width limit of a structure to be formatted into single line if `allow_inline_style` is true.
                if config.structures.small_structures_single_line {
                    if let Some(field_width) = field_width {
                        if field_width > self.width_heuristics.structure_field_width {
                            self.code_line.update_line_style(LineStyle::Multiline)
                        }
                    }
                    if self.width > config.whitespace.max_width
                        || body_width > self.width_heuristics.structure_lit_width
                    {
                        self.code_line.update_line_style(LineStyle::Multiline)
                    } else {
                        self.code_line.update_line_style(LineStyle::Inline)
                    }
                } else {
                    self.code_line.update_line_style(LineStyle::Multiline)
                }
            }
            ExprKind::Collection => {
                if self.width > config.whitespace.max_width
                    || body_width > self.width_heuristics.collection_width
                {
                    self.code_line.update_line_style(LineStyle::Multiline)
                } else {
                    self.code_line.update_line_style(LineStyle::Normal)
                }
            }
            _ => self.code_line.update_line_style(LineStyle::default()),
        }
    }
    /// Update `Shape::code_line` with the provided settings.
    pub(crate) fn update_line_settings(&mut self, line_settings: CodeLine) {
        self.code_line = line_settings
    }
    /// Reset `Shape::code_line` to default.
    pub(crate) fn reset_line_settings(&mut self) {
        self.update_line_settings(CodeLine::default())
    }
    /// Update the value of `has_where_clause`.
    pub(crate) fn update_where_clause(&mut self) {
        match self.has_where_clause {
            true => self.has_where_clause = false,
            false => self.has_where_clause = true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fmt::Formatter;

    #[test]
    fn indent_add_sub() {
        let indent = Indent::new(4) + Indent::new(8);
        assert_eq!(12, indent.block_indent);

        let indent = indent - Indent::new(4);
        assert_eq!(8, indent.block_indent);
    }
    #[test]
    fn indent_to_string_spaces() {
        let mut formatter = Formatter::default();
        formatter.shape.indent = Indent::new(12);

        // 12 spaces
        assert_eq!(
            "            ",
            formatter.shape.indent.to_string(&formatter.config).unwrap()
        );
    }

    #[test]
    fn indent_to_string_hard_tabs() {
        let mut formatter = Formatter::default();
        formatter.config.whitespace.hard_tabs = true;
        formatter.shape.indent = Indent::new(8);

        // 2 tabs + 4 spaces
        assert_eq!(
            "\t\t",
            formatter.shape.indent.to_string(&formatter.config).unwrap()
        );
    }

    #[test]
    fn shape_block_indent() {
        let mut formatter = Formatter::default();
        formatter.config.whitespace.tab_spaces = 24;
        let max_width = formatter.config.whitespace.max_width;
        formatter.shape.update_width(max_width);
        formatter.shape.block_indent(&formatter.config);

        assert_eq!(max_width, formatter.shape.width);
        assert_eq!(24, formatter.shape.indent.block_indent);
    }

    #[test]
    fn test_get_line_style_struct() {
        let mut formatter = Formatter::default();
        formatter.shape.code_line.update_expr_kind(ExprKind::Struct);
        formatter
            .shape
            .get_line_style(Some(9), 18, &formatter.config);
        assert_eq!(LineStyle::Inline, formatter.shape.code_line.line_style);

        formatter
            .shape
            .get_line_style(Some(10), 19, &formatter.config);
        assert_eq!(LineStyle::Multiline, formatter.shape.code_line.line_style);
    }

    #[test]
    fn test_reset_line_style() {
        let mut formatter = Formatter::default();
        formatter.shape.code_line.update_expr_kind(ExprKind::Struct);
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Inline);

        formatter
            .shape
            .get_line_style(Some(8), 18, &formatter.config);
        assert_eq!(LineStyle::Inline, formatter.shape.code_line.line_style);

        formatter.shape.reset_line_settings();
        assert_eq!(LineStyle::Normal, formatter.shape.code_line.line_style);
    }
}
