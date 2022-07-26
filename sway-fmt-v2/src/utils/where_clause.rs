use crate::fmt::*;
use std::fmt::Write;
use sway_parse::{WhereBound, WhereClause};
use sway_types::Spanned;

impl Format for WhereClause {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(
            formatted_code,
            "\n{}{}",
            &formatter.shape.to_string(formatter)?,
            self.where_token.span().as_str(),
        )?;
        let mut shape = formatter.shape;
        shape = shape.block_indent(formatter);
        formatter.shape = shape;
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
        shape = shape.block_unindent(formatter);
        formatter.shape = shape;
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
            "{}{}{} {}",
            &formatter.shape.to_string(formatter)?, // `Indent`
            self.ty_name.span().as_str(),           // `Ident`
            self.colon_token.span().as_str(),       // `ColonToken`
            self.bounds.span().as_str()             //  TODO: `Traits`
        )?;
        Ok(())
    }
}
