use crate::{
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {close_angle_bracket, open_angle_bracket},
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
        if let Some((qualified_path_root, double_colon_token)) = &self.root_opt {
            if let Some(root) = &qualified_path_root {
                open_angle_bracket(formatted_code)?;
                root.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                close_angle_bracket(formatted_code)?;
            }
            write!(formatted_code, "{}", double_colon_token.ident().as_str())?;
        }
        self.prefix.format(formatted_code, formatter)?;
        for (double_colon_token, path_expr_segment) in self.suffix.iter() {
            write!(formatted_code, "{}", double_colon_token.span().as_str())?;
            path_expr_segment.format(formatted_code, formatter)?;
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
        if let Some((double_colon_token, generic_args)) = &self.generics_opt {
            write!(formatted_code, "{}", double_colon_token.span().as_str())?;
            generic_args.format(formatted_code, formatter)?;
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
        if let Some((as_token, path_type)) = &self.as_trait {
            write!(formatted_code, " {} ", as_token.span().as_str())?;
            path_type.format(formatted_code, formatter)?;
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
        for (double_colon_token, path_type_segment) in self.suffix.iter() {
            write!(formatted_code, "{}", double_colon_token.span().as_str())?;
            path_type_segment.format(formatted_code, formatter)?;
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
        if let Some((double_colon_opt, generic_args)) = &self.generics_opt {
            if let Some(double_colon_token) = &double_colon_opt {
                write!(formatted_code, "{}", double_colon_token.span().as_str())?;
            }
            generic_args.format(formatted_code, formatter)?;
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
