use crate::{
    fmt::*,
    utils::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{WhereBound, WhereClause};
use sway_types::Spanned;

impl Format for WhereClause {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            formatted_code,
            "{}{}",
            &formatter.shape.indent.to_string(&formatter.config)?,
            self.where_token.span().as_str(),
        )?;
        formatter.shape.block_indent(&formatter.config);
        // We should add a multiline field to `Shape`
        // so we can reduce this code block to:
        //
        // ```rust,ignore
        // self.bounds.format(formatted_code, formatter)?;
        // ```
        //
        let value_pairs = self.bounds.value_separator_pairs.clone();
        for pair in value_pairs.iter() {
            // `WhereBound`
            pair.0.format(formatted_code, formatter)?;
            // `CommaToken`
            writeln!(formatted_code, "{}", pair.1.span().as_str())?;
        }
        // reset indent
        formatter.shape.block_unindent(&formatter.config);
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
            &formatter.shape.indent.to_string(&formatter.config)?, // `Indent`
            self.ty_name.span().as_str(),                          // `Ident`
            self.colon_token.span().as_str(),                      // `ColonToken`
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
