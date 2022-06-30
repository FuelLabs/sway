use crate::fmt::*;
use std::fmt::Write;
use sway_parse::{PathExpr, PathExprSegment, PathType, PathTypeSegment, QualifiedPathRoot};
use sway_types::Spanned;

impl Format for PathExpr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if let Some(root_opt) = &self.root_opt {
            if let Some(root) = &root_opt.0 {
                formatted_code.push_str(root.open_angle_bracket_token.span().as_str());
                root.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                formatted_code.push_str(root.close_angle_bracket_token.span().as_str());
            }
            formatted_code.push_str(root_opt.1.span().as_str());
        }
        self.prefix.format(formatted_code, formatter)?;
        for suffix in self.suffix.iter() {
            formatted_code.push_str(suffix.0.span().as_str());
            suffix.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for PathExprSegment {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `~`
        if let Some(tilde) = &self.fully_qualified {
            formatted_code.push_str(tilde.span().as_str());
        }
        // name
        formatted_code.push_str(self.name.span().as_str());
        // generics `::<args>`
        if let Some(generic_args) = &self.generics_opt {
            formatted_code.push_str(generic_args.0.span().as_str());
            generic_args.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for QualifiedPathRoot {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Commenting this out since this will err until `Ty` formatting gets in
        // self.ty.format(formatted_code, formatter)?;
        if let Some(as_trait) = &self.as_trait {
            write!(formatted_code, " {} ", as_trait.0.span().as_str())?;
            as_trait.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for PathType {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if let Some(root_opt) = &self.root_opt {
            if let Some(root) = &root_opt.0 {
                formatted_code.push_str(root.open_angle_bracket_token.span().as_str());
                root.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                formatted_code.push_str(root.close_angle_bracket_token.span().as_str());
            }
            formatted_code.push_str(root_opt.1.span().as_str());
        }
        self.prefix.format(formatted_code, formatter)?;
        for suffix in self.suffix.iter() {
            formatted_code.push_str(suffix.0.span().as_str());
            suffix.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for PathTypeSegment {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // `~`
        if let Some(tilde) = &self.fully_qualified {
            formatted_code.push_str(tilde.span().as_str());
        }
        // name
        formatted_code.push_str(self.name.span().as_str());
        // generics `::<args>`
        if let Some(generic_args) = &self.generics_opt {
            if let Some(double_colon) = &generic_args.0 {
                formatted_code.push_str(double_colon.span().as_str());
            }
            generic_args.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}
