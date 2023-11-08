use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{WhereBound, WhereClause};
use sway_types::{ast::PunctKind, Spanned};

impl Format for WhereClause {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            formatted_code,
            "{}{}",
            &formatter.indent_to_str()?,
            self.where_token.span().as_str(),
        )?;
        formatter.indent();
        // We should add a multiline field to `Shape`
        // so we can reduce this code block to:
        //
        // ```rust,ignore
        // self.bounds.format(formatted_code, formatter)?;
        // ```
        //
        let value_pairs = self.bounds.value_separator_pairs.clone();
        for (bound, comma_token) in value_pairs.iter() {
            // `WhereBound`
            bound.format(formatted_code, formatter)?;
            // `CommaToken`
            writeln!(formatted_code, "{}", comma_token.span().as_str())?;
        }
        if let Some(final_value) = &self.bounds.final_value_opt {
            final_value.format(formatted_code, formatter)?;
            writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
        }
        // reset indent
        formatter.unindent();
        Ok(())
    }
}

impl Format for WhereBound {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{}{} ",
            &formatter.indent_to_str()?,      // `Indent`
            self.ty_name.span().as_str(),     // `Ident`
            self.colon_token.span().as_str(), // `ColonToken`
        )?;
        self.bounds.format(formatted_code, formatter)?;
        Ok(())
    }
}

impl LeafSpans for WhereBound {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.ty_name.leaf_spans();
        collected_spans.append(&mut self.colon_token.leaf_spans());
        collected_spans.append(&mut self.bounds.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for WhereClause {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.where_token.span())];
        collected_spans.append(&mut self.bounds.leaf_spans());
        collected_spans
    }
}
