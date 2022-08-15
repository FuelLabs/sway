use crate::{
    fmt::*,
    utils::{
        bracket::{close_angle_bracket, open_angle_bracket},
        comments::{ByteSpan, LeafSpans},
    },
};
use std::{fmt::Write, vec};
use sway_ast::{
    keywords::Token, PathExpr, PathExprSegment, PathType, PathTypeSegment, QualifiedPathRoot,
};
use sway_types::Spanned;

impl Format for PathExpr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if let Some((root, double_colon_token)) = &self.root_opt {
            if let Some(root) = &root {
                open_angle_bracket(formatted_code)?;
                root.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                close_angle_bracket(formatted_code)?;
            }
            write!(formatted_code, "{}", double_colon_token.ident().as_str())?;
        }
        self.prefix.format(formatted_code, formatter)?;
        for suffix in self.suffix.iter() {
            write!(formatted_code, "{}", suffix.0.span().as_str())?;
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
            write!(formatted_code, "{}", tilde.span().as_str())?;
        }
        // name
        write!(formatted_code, "{}", self.name.span().as_str())?;
        // generics `::<args>`
        if let Some(generic_args) = &self.generics_opt {
            write!(formatted_code, "{}", generic_args.0.span().as_str())?;
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
        self.ty.format(formatted_code, formatter)?;
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
                open_angle_bracket(formatted_code)?;
                root.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                close_angle_bracket(formatted_code)?;
            }
            write!(formatted_code, "{}", root_opt.1.span().as_str())?;
        }
        self.prefix.format(formatted_code, formatter)?;
        for suffix in self.suffix.iter() {
            write!(formatted_code, "{}", suffix.0.span().as_str())?;
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
            write!(formatted_code, "{}", tilde.span().as_str())?;
        }
        // name
        write!(formatted_code, "{}", self.name.span().as_str())?;
        // generics `::<args>`
        if let Some(generic_args) = &self.generics_opt {
            if let Some(double_colon) = &generic_args.0 {
                write!(formatted_code, "{}", double_colon.span().as_str())?;
            }
            generic_args.1.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl LeafSpans for PathExpr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for PathType {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}
