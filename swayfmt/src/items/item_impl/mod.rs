use crate::{
    comments::rewrite_with_comments,
    config::items::ItemBraceStyle,
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{ItemImpl, ItemImplItem};
use sway_types::{ast::Delimiter, Spanned};

#[cfg(test)]
mod tests;

impl Format for ItemImpl {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        write!(formatted_code, "{}", self.impl_token.span().as_str())?;
        if let Some(generic_params) = &self.generic_params_opt {
            generic_params.format(formatted_code, formatter)?;
        }
        write!(formatted_code, " ")?;
        if let Some((path_type, for_token)) = &self.trait_opt {
            path_type.format(formatted_code, formatter)?;
            write!(formatted_code, " {} ", for_token.span().as_str())?;
        }
        self.ty.format(formatted_code, formatter)?;
        if let Some(where_clause) = &self.where_clause_opt {
            write!(formatted_code, " ")?;
            where_clause.format(formatted_code, formatter)?;
            formatter.shape.code_line.update_where_clause(true);
        }
        Self::open_curly_brace(formatted_code, formatter)?;
        let contents = self.contents.get();
        for item in contents.iter() {
            write!(formatted_code, "{}", formatter.indent_str()?,)?;
            item.format(formatted_code, formatter)?;
            writeln!(formatted_code)?;
        }
        Self::close_curly_brace(formatted_code, formatter)?;

        rewrite_with_comments::<ItemImpl>(
            formatter,
            self.span(),
            self.leaf_spans(),
            formatted_code,
            start_len,
        )?;

        Ok(())
    }
}

impl Format for ItemImplItem {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            ItemImplItem::Fn(fn_decl) => fn_decl.format(formatted_code, formatter),
            ItemImplItem::Const(const_decl) => const_decl.format(formatted_code, formatter),
            ItemImplItem::Type(type_decl) => type_decl.format(formatted_code, formatter),
        }
    }
}

impl CurlyBrace for ItemImpl {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        formatter.indent();
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                writeln!(line, "\n{open_brace}")?;
            }
            ItemBraceStyle::SameLineWhere => match formatter.shape.code_line.has_where_clause {
                true => {
                    writeln!(line, "{open_brace}")?;
                    formatter.shape.code_line.update_where_clause(false);
                }
                false => {
                    writeln!(line, " {open_brace}")?;
                }
            },
            _ => {
                // TODO: implement PreferSameLine
                writeln!(line, " {open_brace}")?;
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.unindent();
        write!(
            line,
            "{}{}",
            formatter.indent_str()?,
            Delimiter::Brace.as_close_char()
        )?;

        Ok(())
    }
}

impl LeafSpans for ItemImplItem {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![];
        match self {
            ItemImplItem::Fn(fn_decl) => collected_spans.append(&mut fn_decl.leaf_spans()),
            ItemImplItem::Const(const_decl) => collected_spans.append(&mut const_decl.leaf_spans()),
            ItemImplItem::Type(type_decl) => collected_spans.append(&mut type_decl.leaf_spans()),
        }
        collected_spans
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
