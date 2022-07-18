use crate::{
    fmt::{Format, FormattedCode, Formatter, FormatterError},
    utils::comments::{CommentSpan, CommentVisitor},
};
use std::fmt::Write;
use sway_parse::{
    brackets::SquareBrackets,
    expr::Expr,
    keywords::{StrToken, UnderscoreToken},
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
                arr_descriptor
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
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
                tup_descriptor
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
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
        // TODO: once expr formatting is completly implemented switch this to use the actual formatting rather than the raw str coming from span
        write!(
            formatted_code,
            "{} {}",
            self.semicolon_token.span().as_str(),
            self.length.span().as_str()
        )?;
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
            head.format(formatted_code, formatter)?;
            write!(formatted_code, "{} ", comma_token.ident().as_str())?;
            tail.format(formatted_code, formatter)?;
        }
        Ok(())
    }
}

impl CommentVisitor for Ty {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        match self {
            Ty::Path(path) => path.collect_spans(),
            Ty::Tuple(tuple) => tuple.collect_spans(),
            Ty::Array(array) => array.collect_spans(),
            Ty::Str { str_token, length } => {
                let mut collected_spans = vec![CommentSpan::from_span(str_token.span())];
                collected_spans.append(&mut length.collect_spans());
                collected_spans
            }
            Ty::Infer { underscore_token } => vec![CommentSpan::from_span(underscore_token.span())],
        }
    }
}

impl CommentVisitor for TyTupleDescriptor {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        if let TyTupleDescriptor::Cons {
            head,
            comma_token,
            tail,
        } = self
        {
            collected_spans.append(&mut head.collect_spans());
            collected_spans.push(CommentSpan::from_span(comma_token.span()));
            collected_spans.append(&mut tail.collect_spans());
        }
        collected_spans
    }
}

impl CommentVisitor for TyArrayDescriptor {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.append(&mut self.ty.collect_spans());
        collected_spans.push(CommentSpan::from_span(self.semicolon_token.span()));
        collected_spans.append(&mut self.length.collect_spans());
        collected_spans
    }
}
