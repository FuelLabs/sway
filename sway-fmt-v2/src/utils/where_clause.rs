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
            &formatter.shape.indent.to_string(formatter),
            self.where_token.span().as_str(),
        )?;
        // indent right
        formatter.shape = formatter
            .shape
            .block_indent(formatter.config.whitespace.tab_spaces);
        // We need to add the `WhereBound` formatting to punctuated so that we
        // can replace the formatting here with:
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
        formatter.shape = formatter
            .shape
            .shrink_left(formatter.config.whitespace.tab_spaces)
            .unwrap_or_default();
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
            &formatter.shape.indent.to_string(formatter), // `Indent`
            self.ty_name.as_str(),                        // `Ident`
            self.colon_token.span().as_str(),             // `ColonToken`
            self.bounds.span().as_str()                   //  TODO: `Traits`
        )?;
        Ok(())
    }
}
