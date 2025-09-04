use crate::{
    comments::{has_comments_in_formatter, rewrite_with_comments, write_comments},
    constants::NEW_LINE,
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        CurlyBrace,
    },
};
use std::fmt::Write;
use sway_ast::{
    keywords::{ForToken, ImplToken, Keyword},
    ItemImpl, ItemImplItem,
};
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

        write!(formatted_code, "{}", ImplToken::AS_STR)?;
        if let Some(generic_params) = &self.generic_params_opt {
            generic_params.format(formatted_code, formatter)?;
        }
        write!(formatted_code, " ")?;
        if let Some((path_type, _for_token)) = &self.trait_opt {
            path_type.format(formatted_code, formatter)?;
            write!(formatted_code, " {} ", ForToken::AS_STR)?;
        }
        self.ty.format(formatted_code, formatter)?;
        if let Some(where_clause) = &self.where_clause_opt {
            writeln!(formatted_code)?;
            where_clause.format(formatted_code, formatter)?;
            formatter.shape.code_line.update_where_clause(true);
        }

        let contents = self.contents.get();
        if contents.is_empty() {
            let range = self.span().into();
            Self::open_curly_brace(formatted_code, formatter)?;
            if has_comments_in_formatter(formatter, &range) {
                formatter.indent();
                write_comments(formatted_code, range, formatter)?;
            }
            Self::close_curly_brace(formatted_code, formatter)?;
        } else {
            Self::open_curly_brace(formatted_code, formatter)?;
            formatter.indent();
            write!(formatted_code, "{NEW_LINE}")?;
            for item in contents.iter() {
                item.format(formatted_code, formatter)?;
                write!(formatted_code, "{NEW_LINE}")?;
            }
            Self::close_curly_brace(formatted_code, formatter)?;
        }

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
        let open_brace = Delimiter::Brace.as_open_char();
        match formatter.shape.code_line.has_where_clause {
            true => {
                write!(line, "{open_brace}")?;
                formatter.shape.code_line.update_where_clause(false);
            }
            false => {
                write!(line, " {open_brace}")?;
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
            formatter.indent_to_str()?,
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
