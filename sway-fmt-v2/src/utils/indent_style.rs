//! Associated functions and tests for handling of indentation.
use std::{
    borrow::Cow,
    cmp::min,
    ops::{Add, Sub},
};

use crate::{
    constants::{INDENT_BUFFER, INDENT_BUFFER_LEN, INFINITE_SHAPE_WIDTH},
    fmt::Formatter,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Indent {
    /// Width of the block indent, in characters. Must be a multiple of
    /// Config::tab_spaces.
    pub(crate) block_indent: usize,
    /// Alignment in characters.
    pub(crate) alignment: usize,
}

impl Indent {
    pub(crate) fn new(block_indent: usize, alignment: usize) -> Self {
        Self {
            block_indent,
            alignment,
        }
    }

    pub(crate) fn from_width(formatter: &Formatter, width: usize) -> Self {
        let tab_spaces = formatter.config.whitespace.tab_spaces;
        if formatter.config.whitespace.hard_tabs {
            let tab_num = width / tab_spaces;
            let alignment = width % tab_spaces;
            Self::new(tab_spaces * tab_num, alignment)
        } else {
            Self::new(width, 0)
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(0, 0)
    }

    pub(crate) fn block_only(&self) -> Self {
        Self {
            block_indent: self.block_indent,
            alignment: 0,
        }
    }

    pub(crate) fn block_indent(mut self, formatter: &Formatter) -> Self {
        self.block_indent += formatter.config.whitespace.tab_spaces;
        self
    }

    pub(crate) fn block_unindent(mut self, formatter: &Formatter) -> Self {
        let tab_spaces = formatter.config.whitespace.tab_spaces;
        if self.block_indent < tab_spaces {
            Indent::new(self.block_indent, 0)
        } else {
            self.block_indent -= tab_spaces;
            self
        }
    }

    pub(crate) fn width(&self) -> usize {
        self.block_indent + self.alignment
    }

    pub(crate) fn to_string(self, formatter: &Formatter) -> Cow<'static, str> {
        self.to_string_inner(formatter, 1)
    }

    pub(crate) fn to_string_with_newline(self, formatter: &Formatter) -> Cow<'static, str> {
        self.to_string_inner(formatter, 0)
    }

    fn to_string_inner(self, formatter: &Formatter, offset: usize) -> Cow<'static, str> {
        let (num_tabs, num_spaces) = if formatter.config.whitespace.hard_tabs {
            (
                self.block_indent / formatter.config.whitespace.tab_spaces,
                self.alignment,
            )
        } else {
            (0, self.width())
        };
        let num_chars = num_tabs + num_spaces;
        if num_tabs == 0 && num_chars + offset <= INDENT_BUFFER_LEN {
            Cow::from(&INDENT_BUFFER[offset..=num_chars])
        } else {
            let mut indent = String::with_capacity(num_chars + if offset == 0 { 1 } else { 0 });
            if offset == 0 {
                indent.push('\n');
            }
            for _ in 0..num_tabs {
                indent.push('\t')
            }
            for _ in 0..num_spaces {
                indent.push(' ')
            }
            Cow::from(indent)
        }
    }
}

impl Add for Indent {
    type Output = Indent;

    fn add(self, rhs: Indent) -> Indent {
        Indent {
            block_indent: self.block_indent + rhs.block_indent,
            alignment: self.alignment + rhs.alignment,
        }
    }
}

impl Sub for Indent {
    type Output = Indent;

    fn sub(self, rhs: Indent) -> Indent {
        Indent::new(
            self.block_indent - rhs.block_indent,
            self.alignment - rhs.alignment,
        )
    }
}

impl Add<usize> for Indent {
    type Output = Indent;

    fn add(self, rhs: usize) -> Indent {
        Indent::new(self.block_indent, self.alignment + rhs)
    }
}

impl Sub<usize> for Indent {
    type Output = Indent;

    fn sub(self, rhs: usize) -> Indent {
        Indent::new(self.block_indent, self.alignment - rhs)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Shape {
    pub(crate) width: usize,
    // The current indentation of code.
    pub(crate) indent: Indent,
    // Indentation + any already emitted text on the first line of the current
    // statement.
    pub(crate) offset: usize,
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
    pub(crate) fn legacy(width: usize, indent: Indent) -> Self {
        Self {
            width,
            indent,
            offset: indent.alignment,
        }
    }

    pub(crate) fn indented(indent: Indent, formatter: &Formatter) -> Self {
        Self {
            width: formatter
                .config
                .whitespace
                .max_width
                .saturating_sub(indent.width()),
            indent,
            offset: indent.alignment,
        }
    }

    pub(crate) fn with_max_width(&self, formatter: &Formatter) -> Self {
        Self {
            width: formatter
                .config
                .whitespace
                .max_width
                .saturating_sub(self.indent.width()),
            ..*self
        }
    }

    pub(crate) fn visual_indent(&self, extra_width: usize) -> Self {
        let alignment = self.offset + extra_width;
        Self {
            width: self.width,
            indent: Indent::new(self.indent.block_indent, alignment),
            offset: alignment,
        }
    }

    pub(crate) fn block_indent(&self, extra_width: usize) -> Self {
        if self.indent.alignment == 0 {
            Self {
                width: self.width,
                indent: Indent::new(self.indent.block_indent + extra_width, 0),
                offset: 0,
            }
        } else {
            Self {
                width: self.width,
                indent: self.indent + extra_width,
                offset: self.indent.alignment + extra_width,
            }
        }
    }

    pub(crate) fn block_left(&self, width: usize) -> Option<Self> {
        self.block_indent(width).sub_width(width)
    }

    pub(crate) fn add_offset(&self, extra_width: usize) -> Self {
        Self {
            offset: self.offset + extra_width,
            ..*self
        }
    }

    pub(crate) fn block(&self) -> Self {
        Self {
            indent: self.indent.block_only(),
            ..*self
        }
    }

    pub(crate) fn saturating_sub_width(&self, width: usize) -> Self {
        self.sub_width(width).unwrap_or(Self { width: 0, ..*self })
    }

    pub(crate) fn sub_width(&self, width: usize) -> Option<Self> {
        Some(Self {
            width: self.width.checked_sub(width)?,
            ..*self
        })
    }

    pub(crate) fn shrink_left(&self, width: usize) -> Option<Self> {
        Some(Self {
            width: self.width.checked_sub(width)?,
            indent: self.indent + width,
            offset: self.offset + width,
        })
    }

    pub(crate) fn offset_left(&self, width: usize) -> Option<Self> {
        self.add_offset(width).sub_width(width)
    }

    pub(crate) fn used_width(&self) -> usize {
        self.indent.block_indent + self.offset
    }

    pub(crate) fn rhs_overhead(&self, formatter: &Formatter) -> usize {
        formatter
            .config
            .whitespace
            .max_width
            .saturating_sub(self.used_width() + self.width)
    }

    pub(crate) fn comment(&self, formatter: &Formatter) -> Self {
        let width = min(
            self.width,
            formatter
                .config
                .comments
                .comment_width
                .saturating_sub(self.indent.width()),
        );
        Self { width, ..*self }
    }

    pub(crate) fn to_string_with_newline(self, formatter: &Formatter) -> Cow<'static, str> {
        let mut offset_indent = self.indent;
        offset_indent.alignment = self.offset;
        offset_indent.to_string_inner(formatter, 0)
    }

    /// Creates a `Shape` with a virtually infinite width.
    pub(crate) fn infinite_width(&self) -> Self {
        Self {
            width: INFINITE_SHAPE_WIDTH,
            ..*self
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indent_add_sub() {
        let indent = Indent::new(4, 8) + Indent::new(8, 12);
        assert_eq!(12, indent.block_indent);
        assert_eq!(20, indent.alignment);

        let indent = indent - Indent::new(4, 4);
        assert_eq!(8, indent.block_indent);
        assert_eq!(16, indent.alignment);
    }

    #[test]
    fn indent_add_sub_alignment() {
        let indent = Indent::new(4, 8) + 4;
        assert_eq!(4, indent.block_indent);
        assert_eq!(12, indent.alignment);

        let indent = indent - 4;
        assert_eq!(4, indent.block_indent);
        assert_eq!(8, indent.alignment);
    }

    #[test]
    fn indent_to_string_spaces() {
        let formatter = Formatter::default();
        let indent = Indent::new(4, 8);

        // 12 spaces
        assert_eq!("            ", indent.to_string(&formatter));
    }

    #[test]
    fn indent_to_string_hard_tabs() {
        let mut formatter = Formatter::default();
        formatter.config.whitespace.hard_tabs = true;
        let indent = Indent::new(8, 4);

        // 2 tabs + 4 spaces
        assert_eq!("\t\t    ", indent.to_string(&formatter));
    }

    #[test]
    fn shape_visual_indent() {
        let formatter = Formatter::default();
        let max_width = formatter.config.whitespace.max_width;
        let indent = Indent::new(4, 8);
        let shape = Shape::legacy(max_width, indent);
        let shape = shape.visual_indent(20);

        assert_eq!(max_width, shape.width);
        assert_eq!(4, shape.indent.block_indent);
        assert_eq!(28, shape.indent.alignment);
        assert_eq!(28, shape.offset);
    }

    #[test]
    fn shape_block_indent_without_alignment() {
        let formatter = Formatter::default();
        let max_width = formatter.config.whitespace.max_width;
        let indent = Indent::new(4, 0);
        let shape = Shape::legacy(max_width, indent);
        let shape = shape.block_indent(20);

        assert_eq!(max_width, shape.width);
        assert_eq!(24, shape.indent.block_indent);
        assert_eq!(0, shape.indent.alignment);
        assert_eq!(0, shape.offset);
    }

    #[test]
    fn shape_block_indent_with_alignment() {
        let formatter = Formatter::default();
        let max_width = formatter.config.whitespace.max_width;
        let indent = Indent::new(4, 8);
        let shape = Shape::legacy(max_width, indent);
        let shape = shape.block_indent(20);

        assert_eq!(max_width, shape.width);
        assert_eq!(4, shape.indent.block_indent);
        assert_eq!(28, shape.indent.alignment);
        assert_eq!(28, shape.offset);
    }
}
