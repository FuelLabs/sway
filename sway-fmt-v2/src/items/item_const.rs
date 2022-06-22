use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemConst;
use sway_types::Spanned;

impl Format for ItemConst {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        // TODO: creating this formatted_code with String::new() will likely cause lots of
        // reallocations maybe we can explore how we can do this, starting with with_capacity may help.
        let mut formatted_code = String::new();

        // Check if visibility token exists if so add it.
        if let Some(visibility_token) = &self.visibility {
            formatted_code.push_str(visibility_token.span().as_str());
            formatted_code.push(' ');
        }

        // Add the const token
        formatted_code.push_str(self.const_token.span().as_str());
        formatted_code.push(' ');

        // Add name of the const
        formatted_code.push_str(self.name.as_str());

        // Check if ty exists
        if let Some(ty) = &self.ty_opt {
            // Add colon
            formatted_code.push_str(ty.0.span().as_str());
            // TODO: We are not applying any custom formatting to ty probably we will need to in the future.
            // Add ty
            formatted_code.push(' ');
            formatted_code.push_str(ty.1.span().as_str());
        }

        formatted_code.push(' ');
        // Add equal token
        formatted_code.push_str(self.eq_token.ident().as_str());
        formatted_code.push(' ');

        // TODO: We are not applying any custom formatting to expr, probably we will need to in the future.
        formatted_code.push_str(self.expr.span().as_str());
        formatted_code.push_str(self.semicolon_token.ident().as_str());
        formatted_code
    }
}
