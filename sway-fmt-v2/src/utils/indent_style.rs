//! Associated functions and tests for handling of indentation.
use std::{
    borrow::Cow,
    fmt::Write,
    ops::{Add, Sub},
};

use crate::{
    constants::{HARD_TAB, INDENT_BUFFER, INDENT_BUFFER_LEN},
    fmt::Formatter,
    FormatterError,
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
    fn block_indent(mut self, formatter: &Formatter) -> Self {
        self.block_indent += formatter.config.whitespace.tab_spaces;
        self
    }
    /// Removes a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    /// If the current level of indentation would be negative, a new `Indent` is contructed.
    fn block_unindent(mut self, formatter: &Formatter) -> Self {
        let tab_spaces = formatter.config.whitespace.tab_spaces;
        if self.block_indent < tab_spaces {
            Self::new(self.block_indent)
        } else {
            self.block_indent -= tab_spaces;
            self
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
        formatter: &Formatter,
        offset: usize,
    ) -> Result<Cow<'static, str>, FormatterError> {
        let (num_tabs, num_spaces) = if formatter.config.whitespace.hard_tabs {
            (
                self.block_indent / formatter.config.whitespace.tab_spaces,
                0,
            )
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum LineStyle {
    Inline,
    Multiline,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self::Multiline
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct Shape {
    /// The current number of characters in the given `Item`.
    pub(crate) width: usize,
    /// The current indentation of code.
    pub(crate) indent: Indent,
    /// Determines whether a code line is inline, or multiline.
    pub(crate) line_style: LineStyle,
    /// Used in determining `SameLineWhere` formatting.
    /// Default is false.
    pub(crate) has_where_clause: bool,
}

impl Shape {
    /// `indent` is the indentation of the first line. The next lines
    /// should begin with at least `indent` spaces (except backwards
    /// indentation). The first line should not begin with indentation.
    /// `width` is the maximum number of characters on the last line
    /// (excluding `indent`). The width of other lines is not limited by
    /// `width`.
    ///
    /// Note that in reality, we sometimes use width for lines other than the
    /// last (i.e., we are conservative).
    // .......*-------*
    //        |       |
    //        |     *-*
    //        *-----|
    // |<------------>|  max width
    // |<---->|          indent
    //        |<--->|    width
    //
    /// Construct a new `Shape` with a given width and level of indentation.
    pub(crate) fn legacy(&self, width: usize, indent: Indent) -> Self {
        Self {
            width,
            indent,
            line_style: self.line_style,
            has_where_clause: self.has_where_clause,
        }
    }
    /// Construct a new `Shape` that takes into account the current level of indentation.
    pub(crate) fn indented(&self, indent: Indent, formatter: &Formatter) -> Self {
        Self {
            width: formatter
                .config
                .whitespace
                .max_width
                .saturating_sub(indent.width()),
            indent,
            line_style: self.line_style,
            has_where_clause: self.has_where_clause,
        }
    }
    /// A wrapper for `Indent::block_indent()`.
    ///
    /// Adds a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    pub(crate) fn block_indent(&mut self, formatter: &Formatter) -> Self {
        Self {
            indent: self.indent.block_indent(formatter),
            ..*self
        }
    }
    /// A wrapper for `Indent::block_unindent()`.
    ///
    /// Removes a level of indentation specified by the `Formatter::config` to the current `block_indent`.
    /// If the current level of indentation would be negative, a new `Indent` is contructed.
    pub(crate) fn block_unindent(&self, formatter: &Formatter) -> Self {
        Self {
            indent: self.indent.block_unindent(formatter),
            ..*self
        }
    }
    /// Updates `Shape::width` to the current width of the `Item`.
    pub(crate) fn update_width(&self, len_chars: usize) -> Self {
        Self {
            width: len_chars,
            ..*self
        }
    }
    /// Checks the config, and if `small_structure_single_line` is enabled,
    /// determines whether the `Shape::width` is greater than the `structure_lit_width`
    /// threshold. If it isn't, the `Shape::line_style` is updated to `Inline`.
    pub(crate) fn get_line_style(&mut self, formatter: &Formatter) -> Self {
        let allow_inline_style = formatter.config.structures.small_structures_single_line;
        // Get the width limit of a structure to be formatted into single line if `allow_inline_style` is true.
        if allow_inline_style {
            let width_heuristics = formatter
                .config
                .heuristics
                .heuristics_pref
                .to_width_heuristics(&formatter.config.whitespace);

            if self.width > width_heuristics.structure_lit_width {
                Self {
                    line_style: LineStyle::Multiline,
                    ..*self
                }
            } else {
                Self {
                    line_style: LineStyle::Inline,
                    ..*self
                }
            }
        } else {
            Self {
                line_style: LineStyle::Multiline,
                ..*self
            }
        }
    }
    /// Reset `Shape::line_style` to default.
    pub(crate) fn reset_line_style(&mut self) -> Self {
        Self {
            line_style: LineStyle::default(),
            ..*self
        }
    }
    /// A wrapper for `Indent::to_string_inner()` that does not add a new line.
    ///
    /// Checks for either `hard_tabs` or `tab_spaces` and creates a
    /// buffer of whitespace from the current level of indentation.
    /// This also takes an offset which determines whether to add a new line.
    pub(crate) fn to_string(
        self,
        formatter: &Formatter,
    ) -> Result<Cow<'static, str>, FormatterError> {
        self.indent.to_string_inner(formatter, 1)
    }
    /// A wrapper for `Indent::to_string_inner()` that also adds a new line.
    ///
    /// Checks for either `hard_tabs` or `tab_spaces` and creates a
    /// buffer of whitespace from the current level of indentation.
    /// This also takes an offset which determines whether to add a new line.
    pub(crate) fn to_string_with_newline(
        self,
        formatter: &Formatter,
    ) -> Result<Cow<'static, str>, FormatterError> {
        self.indent.to_string_inner(formatter, 0)
    }
    /// Update the value of `has_where_clause`.
    pub(crate) fn update_where_clause(&mut self) -> Self {
        match self.has_where_clause {
            true => Self {
                has_where_clause: false,
                ..*self
            },
            false => Self {
                has_where_clause: true,
                ..*self
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            formatter.shape.to_string(&formatter).unwrap()
        );
    }

    #[test]
    fn indent_to_string_hard_tabs() {
        let mut formatter = Formatter::default();
        formatter.config.whitespace.hard_tabs = true;
        formatter.shape.indent = Indent::new(8);

        // 2 tabs + 4 spaces
        assert_eq!("\t\t", formatter.shape.to_string(&formatter).unwrap());
    }

    #[test]
    fn shape_block_indent() {
        let mut formatter = Formatter::default();
        formatter.config.whitespace.tab_spaces = 20;
        let max_width = formatter.config.whitespace.max_width;
        let indent = Indent::new(4);
        let mut shape = Shape::legacy(&formatter.shape, max_width, indent);
        let shape = shape.block_indent(&formatter);

        assert_eq!(max_width, shape.width);
        assert_eq!(24, shape.indent.block_indent);
    }

    #[test]
    fn test_line_style() {
        let mut formatter = Formatter::default();
        let mut shape = formatter.shape;
        formatter.shape = shape.get_line_style(&formatter);
        assert_eq!(LineStyle::Inline, formatter.shape.line_style);

        formatter.shape.width = 19;
        shape = formatter.shape;
        formatter.shape = shape.get_line_style(&formatter);
        assert_eq!(LineStyle::Multiline, formatter.shape.line_style);
    }

    #[test]
    fn test_reset_line_style() {
        let mut formatter = Formatter::default();
        formatter.shape.line_style = LineStyle::Inline;
        let mut shape = formatter.shape;

        formatter.shape = shape.get_line_style(&formatter);
        assert_eq!(LineStyle::Inline, formatter.shape.line_style);

        formatter.shape = shape.reset_line_style();
        assert_eq!(LineStyle::Multiline, formatter.shape.line_style);
    }
}
