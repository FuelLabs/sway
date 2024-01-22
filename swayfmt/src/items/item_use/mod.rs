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
use sway_ast::{CommaToken, ItemUse, UseTree};
use sway_types::{
    ast::{Delimiter, PunctKind},
    Spanned,
};

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
                    .update_line_style(LineStyle::Normal);
                format_use_stmt(self, &mut buf, &mut temp_formatter)?;

                let expr_width = buf.chars().count();
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
                // check for only one import
                if imports.inner.value_separator_pairs.is_empty() {
                    if let Some(single_import) = &imports.inner.final_value_opt {
                        single_import.format(formatted_code, formatter)?;
                    }
                } else {
                    Self::open_curly_brace(formatted_code, formatter)?;
                    // sort group imports
                    let imports = imports.get();
                    let value_pairs = &imports.value_separator_pairs;
                    let mut commas: Vec<&CommaToken> = Vec::new();
                    let mut ord_vec: Vec<String> = value_pairs
                        .iter()
                        .map(
                            |(use_tree, comma_token)| -> Result<FormattedCode, FormatterError> {
                                let mut buf = FormattedCode::new();
                                use_tree.format(&mut buf, formatter)?;
                                commas.push(comma_token);
                                Ok(buf)
                            },
                        )
                        .collect::<Result<_, _>>()?;
                    if let Some(final_value) = &imports.final_value_opt {
                        let mut buf = FormattedCode::new();
                        final_value.format(&mut buf, formatter)?;

                        ord_vec.push(buf);
                    }
                    ord_vec.sort_by(|a, b| {
                        if a == b {
                            std::cmp::Ordering::Equal
                        } else if a == "self" || b == "*" {
                            std::cmp::Ordering::Less
                        } else if b == "self" || a == "*" {
                            std::cmp::Ordering::Greater
                        } else {
                            a.to_lowercase().cmp(&b.to_lowercase())
                        }
                    });
                    for (use_tree, comma) in ord_vec.iter_mut().zip(commas.iter()) {
                        write!(use_tree, "{}", comma.span().as_str())?;
                    }

                    match formatter.shape.code_line.line_style {
                        LineStyle::Multiline => {
                            if imports.final_value_opt.is_some() {
                                if let Some(last) = ord_vec.iter_mut().last() {
                                    write!(last, "{}", PunctKind::Comma.as_char())?;
                                }
                            }

                            writeln!(
                                formatted_code,
                                "{}{}",
                                formatter.indent_to_str()?,
                                ord_vec.join(&format!("\n{}", formatter.indent_to_str()?)),
                            )?;
                        }
                        _ => {
                            write!(formatted_code, "{}", ord_vec.join(" "))?;
                        }
                    }
                    Self::close_curly_brace(formatted_code, formatter)?;
                }
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
            Self::Error { .. } => {
                return Err(FormatterError::SyntaxError);
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
                formatter.indent();
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
                formatter.unindent();
                write!(
                    line,
                    "{}{}",
                    formatter.indent_to_str()?,
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
            UseTree::Error { spans } => spans.iter().map(|s| ByteSpan::from(s.clone())).collect(),
        }
    }
}
