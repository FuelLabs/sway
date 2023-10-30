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
            let mut indent = String::with_capacity(num_chars + usize::from(offset == 0));
            if offset == 0 {
                writeln!(indent)?;
            }
            for _ in 0..num_tabs {
                write!(indent, "{HARD_TAB}")?;
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
/// Information about the line of code currently being evaluated.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct CodeLine {
    /// The current number of characters in the given code line.
    pub(crate) width: usize,
    pub(crate) line_style: LineStyle,
    pub(crate) expr_kind: ExprKind,
    /// Used in determining `SameLineWhere` formatting.
    pub(crate) has_where_clause: bool,
    /// Expression is too long to fit in a single line
    pub(crate) expr_new_line: bool,
}

impl CodeLine {
    pub(crate) fn from(line_style: LineStyle, expr_kind: ExprKind) -> Self {
        Self {
            width: Default::default(),
            line_style,
            expr_kind,
            has_where_clause: Default::default(),
            expr_new_line: false,
        }
    }
    pub(crate) fn reset_width(&mut self) {
        self.width = 0;
    }
    pub(crate) fn update_width(&mut self, new_width: usize) {
        self.width = new_width;
    }
    pub(crate) fn add_width(&mut self, extra_width: usize) {
        self.width += extra_width;
    }
    pub(crate) fn sub_width(&mut self, extra_width: usize) {
        self.width -= extra_width;
    }
    /// Update `CodeLine::line_style` with a given LineStyle.
    pub(crate) fn update_line_style(&mut self, line_style: LineStyle) {
        self.line_style = line_style;
    }
    /// Update `CodeLine::expr_kind` with a given ExprKind.
    pub(crate) fn update_expr_kind(&mut self, expr_kind: ExprKind) {
        self.expr_kind = expr_kind;
    }
    /// Update the value of `has_where_clause`.
    pub(crate) fn update_where_clause(&mut self, has_where_clause: bool) {
        self.has_where_clause = has_where_clause;
    }

    pub(crate) fn update_expr_new_line(&mut self, expr_new_line: bool) {
        self.expr_new_line = expr_new_line;
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
    Import,
    Undetermined,
}

impl Default for ExprKind {
    fn default() -> Self {
        Self::Undetermined
    }
}

/// The current shape of the formatter.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Shape {
    /// The current indentation of code.
    pub(crate) indent: Indent,
    /// Used in determining which heuristics settings to use from the config
    /// and whether a code line is normal, inline or multiline.
    pub(crate) code_line: CodeLine,
    /// The definitive width settings from the `Config`.
    pub(crate) width_heuristics: WidthHeuristics,
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
        if width_heuristics != WidthHeuristics::default() {
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
    /// Checks the current status of a `CodeLine` against the `Shape::width_heuristics` to determine which
    /// `LineStyle` should be applied.
    pub(crate) fn get_line_style(
        &mut self,
        field_width: Option<usize>,
        body_width: Option<usize>,
        config: &Config,
    ) {
        match self.code_line.expr_kind {
            ExprKind::Struct => {
                // Get the width limit of a structure to be formatted into single line if `allow_inline_style` is true.
                if config.structures.small_structures_single_line {
                    if self.code_line.width > config.whitespace.max_width
                        || field_width.unwrap_or(0) > self.width_heuristics.structure_field_width
                        || body_width.unwrap_or(0) > self.width_heuristics.structure_lit_width
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
                if self.code_line.width > config.whitespace.max_width
                    || body_width.unwrap_or(0) > self.width_heuristics.collection_width
                {
                    self.code_line.update_line_style(LineStyle::Multiline)
                } else {
                    self.code_line.update_line_style(LineStyle::Normal)
                }
            }
            ExprKind::Import => {
                if self.code_line.width > config.whitespace.max_width {
                    self.code_line.update_line_style(LineStyle::Multiline)
                } else {
                    self.code_line.update_line_style(LineStyle::Normal)
                }
            }
            ExprKind::Function => {
                if self.code_line.width > config.whitespace.max_width
                    || body_width.unwrap_or(0) > self.width_heuristics.fn_call_width
                {
                    self.code_line.update_line_style(LineStyle::Multiline)
                } else {
                    self.code_line.update_line_style(LineStyle::Normal)
                }
            }
            ExprKind::Conditional => {
                if self.code_line.width < self.width_heuristics.single_line_if_else_max_width {
                    self.code_line.update_line_style(LineStyle::Inline)
                } else if body_width.unwrap_or(0) > self.width_heuristics.chain_width {
                    self.code_line.update_line_style(LineStyle::Multiline)
                } else {
                    self.code_line.update_line_style(LineStyle::Normal)
                }
            }
            _ => self.code_line.update_line_style(LineStyle::default()),
        }
    }
    /// Create a new `Shape` with a new `CodeLine` from a given `LineStyle` and `ExprKind`.
    pub(crate) fn with_code_line_from(self, line_style: LineStyle, expr_kind: ExprKind) -> Self {
        Self {
            code_line: CodeLine::from(line_style, expr_kind),
            ..self
        }
    }
    /// Create a new `Shape` with default `CodeLine`.
    pub(crate) fn with_default_code_line(self) -> Self {
        let mut code_line = CodeLine::default();
        code_line.update_expr_new_line(self.code_line.expr_new_line);
        Self { code_line, ..self }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formatter::Formatter;

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
        formatter.shape.code_line.width = max_width;
        formatter.indent();

        assert_eq!(max_width, formatter.shape.code_line.width);
        assert_eq!(24, formatter.shape.indent.block_indent);
    }

    #[test]
    fn test_get_line_style_struct() {
        let mut formatter = Formatter::default();
        formatter.shape.code_line.update_expr_kind(ExprKind::Struct);
        formatter
            .shape
            .get_line_style(Some(9), Some(18), &formatter.config);
        assert_eq!(LineStyle::Inline, formatter.shape.code_line.line_style);

        formatter
            .shape
            .get_line_style(Some(10), Some(19), &formatter.config);
        assert_eq!(LineStyle::Multiline, formatter.shape.code_line.line_style);
    }
}
