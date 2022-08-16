use crate::{
    config::items::ItemBraceStyle,
    fmt::*,
    utils::{
        bracket::CurlyBrace,
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, ItemImpl};
use sway_types::Spanned;

impl Format for ItemImpl {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            self.impl_token.span().as_str()
        )?;
        if let Some(generic_params) = &self.generic_params_opt {
            generic_params.format(formatted_code, formatter)?;
        }
        write!(formatted_code, " ")?;
        if let Some((path_type, for_token)) = &self.trait_opt {
            path_type.format(formatted_code, formatter)?;
            write!(formatted_code, " {}", for_token.span().as_str())?;
        }
        write!(formatted_code, " ")?;
        self.ty.format(formatted_code, formatter)?;
        if let Some(where_clause) = &self.where_clause_opt {
            write!(formatted_code, " ")?;
            where_clause.format(formatted_code, formatter)?;
            formatter.shape.update_where_clause();
        }
        Self::open_curly_brace(formatted_code, formatter)?;
        let contents = self.contents.get();
        for item in contents.iter() {
            item.format(formatted_code, formatter)?;
        }
        Self::close_curly_brace(formatted_code, formatter)?;

        Ok(())
    }
}

impl CurlyBrace for ItemImpl {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
            ItemBraceStyle::SameLineWhere => match formatter.shape.has_where_clause {
                true => {
                    writeln!(line, "{}", open_brace)?;
                    formatter.shape.update_where_clause();
                    formatter.shape.block_indent(&formatter.config);
                }
                false => {
                    writeln!(line, " {}", open_brace)?;
                    formatter.shape.block_indent(&formatter.config);
                }
            },
            _ => {
                // TODO: implement PreferSameLine
                writeln!(line, " {}", open_brace)?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(line, "{}", Delimiter::Brace.as_close_char())?;
        formatter.shape.block_unindent(&formatter.config);
        Ok(())
    }
}

impl LeafSpans for ItemImpl {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.impl_token.span())];
        if let Some(generic) = &self.generic_params_opt {
            collected_spans.push(ByteSpan::from(generic.parameters.span()));
        }
        if let Some(trait_tuple) = &self.trait_opt {
            collected_spans.append(&mut trait_tuple.leaf_spans());
        }
        collected_spans.append(&mut self.ty.leaf_spans());
        if let Some(where_clause) = &self.where_clause_opt {
            collected_spans.append(&mut where_clause.leaf_spans());
        }
        collected_spans.append(&mut self.contents.leaf_spans());
        collected_spans
    }
}
