use crate::{
    fmt::*,
    utils::{
        comments::{ByteSpan, LeafSpans},
        shape::LineStyle,
    },
};
use std::fmt::Write;
use sway_ast::{
    brackets::SquareBrackets,
    expr::Expr,
    keywords::{StrToken, Token, UnderscoreToken},
    token::Delimiter,
    ty::{Ty, TyArrayDescriptor, TyTupleDescriptor},
};
use sway_types::Spanned;

impl Format for Ty {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Array(arr_descriptor) => {
                write!(formatted_code, "{}", Delimiter::Bracket.as_open_char())?;
                arr_descriptor.get().format(formatted_code, formatter)?;
                write!(formatted_code, "{}", Delimiter::Bracket.as_close_char())?;
                Ok(())
            }
            Self::Infer { underscore_token } => format_infer(formatted_code, underscore_token),
            Self::Path(path_ty) => path_ty.format(formatted_code, formatter),
            Self::Str { str_token, length } => {
                format_str(formatted_code, str_token.clone(), length.clone())
            }
            Self::Tuple(tup_descriptor) => {
                write!(formatted_code, "{}", Delimiter::Parenthesis.as_open_char())?;
                tup_descriptor.get().format(formatted_code, formatter)?;
                write!(formatted_code, "{}", Delimiter::Parenthesis.as_close_char())?;
                Ok(())
            }
        }
    }
}

/// Simply inserts a `_` token to the `formatted_code`.
fn format_infer(
    formatted_code: &mut FormattedCode,
    underscore_token: &UnderscoreToken,
) -> Result<(), FormatterError> {
    formatted_code.push_str(underscore_token.ident().as_str());
    Ok(())
}

impl Format for TyArrayDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.ty.format(formatted_code, formatter)?;
        write!(formatted_code, "{} ", self.semicolon_token.span().as_str())?;
        self.length.format(formatted_code, formatter)?;
        Ok(())
    }
}

fn format_str(
    formatted_code: &mut FormattedCode,
    str_token: StrToken,
    length: SquareBrackets<Box<Expr>>,
) -> Result<(), FormatterError> {
    write!(
        formatted_code,
        "{}{}{}{}",
        str_token.span().as_str(),
        Delimiter::Bracket.as_open_char(),
        length.into_inner().span().as_str(),
        Delimiter::Bracket.as_close_char()
    )?;
    Ok(())
}

impl Format for TyTupleDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if let TyTupleDescriptor::Cons {
            head,
            comma_token,
            tail,
        } = self
        {
            let prev_state = formatter.shape.code_line;
            formatter
                .shape
                .code_line
                .update_line_style(LineStyle::Normal);

            head.format(formatted_code, formatter)?;
            write!(formatted_code, "{} ", comma_token.ident().as_str())?;
            tail.format(formatted_code, formatter)?;

            formatter.shape.update_line_settings(prev_state);
        }

        Ok(())
    }
}

impl LeafSpans for Ty {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            Ty::Path(path) => path.leaf_spans(),
            Ty::Tuple(tuple) => tuple.leaf_spans(),
            Ty::Array(array) => array.leaf_spans(),
            Ty::Str { str_token, length } => {
                let mut collected_spans = vec![ByteSpan::from(str_token.span())];
                collected_spans.append(&mut length.leaf_spans());
                collected_spans
            }
            Ty::Infer { underscore_token } => vec![ByteSpan::from(underscore_token.span())],
        }
    }
}

impl LeafSpans for TyTupleDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let TyTupleDescriptor::Cons {
            head,
            comma_token,
            tail,
        } = self
        {
            collected_spans.append(&mut head.leaf_spans());
            collected_spans.push(ByteSpan::from(comma_token.span()));
            collected_spans.append(&mut tail.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for TyArrayDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.append(&mut self.ty.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans.append(&mut self.length.leaf_spans());
        collected_spans
    }
}
