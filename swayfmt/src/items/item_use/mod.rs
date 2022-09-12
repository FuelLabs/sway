use crate::{
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{
    token::{Delimiter, PunctKind},
    ItemUse, UseTree,
};
use sway_types::Spanned;

#[cfg(test)]
mod tests;

impl Format for ItemUse {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.with_shape(
            formatter
                .shape
                .with_code_line_from(LineStyle::Multiline, ExprKind::Import),
            |formatter| -> Result<(), FormatterError> {
                // get the length in chars of the code_line in a single line format,
                // this include the path
                let mut buf = FormattedCode::new();
                let mut temp_formatter = formatter.clone();
                temp_formatter
                    .shape
                    .code_line
                    .with_line_style(LineStyle::Normal);
                format_use_stmt(self, &mut buf, &mut temp_formatter)?;

                let expr_width = buf.chars().count() as usize;
                formatter.shape.code_line.add_width(expr_width);
                formatter
                    .shape
                    .get_line_style(None, None, &formatter.config);

                format_use_stmt(self, formatted_code, formatter)?;

                Ok(())
            },
        )?;

        Ok(())
    }
}

impl Format for UseTree {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Group { imports } => {
                Self::open_curly_brace(formatted_code, formatter)?;
                // sort group imports
                let imports = imports.get();
                let value_pairs = &imports.value_separator_pairs;
                let mut ord_vec: Vec<String> = value_pairs
                    .iter()
                    .map(
                        |(use_tree, comma_token)| -> Result<FormattedCode, FormatterError> {
                            let mut buf = FormattedCode::new();
                            use_tree.format(&mut buf, formatter)?;
                            write!(buf, "{}", comma_token.span().as_str())?;

                            Ok(buf)
                        },
                    )
                    .collect::<Result<_, _>>()?;
                if let Some(final_value) = &imports.final_value_opt {
                    let mut buf = FormattedCode::new();
                    final_value.format(&mut buf, formatter)?;
                    write!(buf, "{}", PunctKind::Comma.as_char())?;

                    ord_vec.push(buf);
                }
                ord_vec.sort_by_key(|x| x.to_lowercase());

                match formatter.shape.code_line.line_style {
                    LineStyle::Multiline => writeln!(
                        formatted_code,
                        "{}{}",
                        formatter.shape.indent.to_string(&formatter.config)?,
                        ord_vec.join(&format!(
                            "\n{}",
                            formatter.shape.indent.to_string(&formatter.config)?
                        ))
                    )?,
                    _ => {
                        let mut import_str = ord_vec.join(" ");
                        if import_str.ends_with(PunctKind::Comma.as_char()) {
                            import_str.pop();
                        }
                        write!(formatted_code, "{}", import_str)?;
                    }
                }
                Self::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Name { name } => write!(formatted_code, "{}", name.span().as_str())?,
            Self::Rename {
                name,
                as_token,
                alias,
            } => {
                write!(
                    formatted_code,
                    "{} {} {}",
                    name.span().as_str(),
                    as_token.span().as_str(),
                    alias.span().as_str()
                )?;
            }
            Self::Glob { star_token } => {
                write!(formatted_code, "{}", star_token.span().as_str())?;
            }
            Self::Path {
                prefix,
                double_colon_token,
                suffix,
            } => {
                write!(
                    formatted_code,
                    "{}{}",
                    prefix.span().as_str(),
                    double_colon_token.span().as_str()
                )?;
                suffix.format(formatted_code, formatter)?;
            }
        }

        Ok(())
    }
}

impl CurlyBrace for UseTree {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.block_indent(&formatter.config);
                writeln!(line, "{}", Delimiter::Brace.as_open_char())?;
            }
            _ => write!(line, "{}", Delimiter::Brace.as_open_char())?,
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.block_unindent(&formatter.config);
                write!(
                    line,
                    "{}{}",
                    formatter.shape.indent.to_string(&formatter.config)?,
                    Delimiter::Brace.as_close_char()
                )?;
            }
            _ => write!(line, "{}", Delimiter::Brace.as_close_char())?,
        }

        Ok(())
    }
}

fn format_use_stmt(
    item_use: &ItemUse,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    if let Some(pub_token) = &item_use.visibility {
        write!(formatted_code, "{} ", pub_token.span().as_str())?;
    }
    write!(formatted_code, "{} ", item_use.use_token.span().as_str())?;
    if let Some(root_import) = &item_use.root_import {
        write!(formatted_code, "{}", root_import.span().as_str())?;
    }
    item_use.tree.format(formatted_code, formatter)?;
    write!(
        formatted_code,
        "{}",
        item_use.semicolon_token.span().as_str()
    )?;

    Ok(())
}

impl LeafSpans for ItemUse {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.use_token.span()));
        if let Some(root_import) = &self.root_import {
            collected_spans.push(ByteSpan::from(root_import.span()));
        }
        collected_spans.append(&mut self.tree.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}

impl LeafSpans for UseTree {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            UseTree::Group { imports } => imports.leaf_spans(),
            UseTree::Name { name } => vec![ByteSpan::from(name.span())],
            UseTree::Rename {
                name,
                as_token,
                alias,
            } => vec![
                ByteSpan::from(name.span()),
                ByteSpan::from(as_token.span()),
                ByteSpan::from(alias.span()),
            ],
            UseTree::Glob { star_token } => vec![ByteSpan::from(star_token.span())],
            UseTree::Path {
                prefix,
                double_colon_token,
                suffix,
            } => {
                let mut collected_spans = vec![ByteSpan::from(prefix.span())];
                collected_spans.push(ByteSpan::from(double_colon_token.span()));
                collected_spans.append(&mut suffix.leaf_spans());
                collected_spans
            }
        }
    }
}
