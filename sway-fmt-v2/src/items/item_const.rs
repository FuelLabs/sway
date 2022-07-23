use crate::{
    fmt::{Format, FormattedCode, Formatter},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::ItemConst;
use sway_types::Spanned;

impl Format for ItemConst {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Check if visibility token exists if so add it.
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }

        // Add the const token
        write!(formatted_code, "{} ", self.const_token.span().as_str())?;

        // Add name of the const
        write!(formatted_code, "{}", self.name.as_str())?;

        // Check if ty exists
        if let Some(ty) = &self.ty_opt {
            // Add colon
            write!(formatted_code, "{} ", ty.0.span().as_str())?;
            ty.1.format(formatted_code, formatter)?;
        }

        // ` = `
        write!(formatted_code, " {} ", self.eq_token.ident().as_str())?;

        // TODO: We are not applying any custom formatting to expr, probably we will need to in the future.
        write!(
            formatted_code,
            "{}{}",
            self.expr.span().as_str(),
            self.semicolon_token.ident().as_str()
        )?;

        Ok(())
    }
}
