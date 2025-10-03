use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    brackets::SquareBrackets,
    expr::Expr,
    keywords::{
        AmpersandToken, BangToken, Keyword, MutToken, PtrToken, SemicolonToken, SliceToken,
        StrToken, Token, UnderscoreToken,
    },
    ty::{Ty, TyArrayDescriptor, TyTupleDescriptor},
    CommaToken,
};
use sway_types::{ast::Delimiter, Spanned};

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
            Self::Infer {
                underscore_token: _,
            } => format_infer(formatted_code),
            Self::Path(path_ty) => path_ty.format(formatted_code, formatter),
            Self::StringSlice(_) => {
                write!(formatted_code, "{}", StrToken::AS_STR)?;
                Ok(())
            }
            Self::StringArray {
                str_token: _,
                length,
            } => format_str(formatted_code, formatter, length.clone()),
            Self::Tuple(tup_descriptor) => {
                write!(formatted_code, "{}", Delimiter::Parenthesis.as_open_char())?;
                tup_descriptor.get().format(formatted_code, formatter)?;
                write!(formatted_code, "{}", Delimiter::Parenthesis.as_close_char())?;
                Ok(())
            }
            Self::Ptr { ptr_token: _, ty } => format_ptr(formatted_code, formatter, ty.clone()),
            Self::Slice { slice_token, ty } => {
                format_slice(formatted_code, formatter, slice_token, ty.clone())
            }
            Self::Ref {
                ampersand_token: _,
                mut_token,
                ty,
            } => format_ref(formatted_code, formatter, mut_token, ty),
            Self::Never { bang_token: _ } => {
                write!(formatted_code, "{}", BangToken::AS_STR)?;
                Ok(())
            }
            Self::Expr(expr) => expr.format(formatted_code, formatter),
        }
    }
}

/// Simply inserts a `_` token to the `formatted_code`.
fn format_infer(formatted_code: &mut FormattedCode) -> Result<(), FormatterError> {
    formatted_code.push_str(UnderscoreToken::AS_STR);

    Ok(())
}

impl Format for TyArrayDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.ty.format(formatted_code, formatter)?;
        write!(formatted_code, "{} ", SemicolonToken::AS_STR)?;
        self.length.format(formatted_code, formatter)?;

        Ok(())
    }
}

fn format_str(
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    length: SquareBrackets<Box<Expr>>,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", StrToken::AS_STR)?;
    write!(formatted_code, "{}", Delimiter::Bracket.as_open_char())?;
    length.into_inner().format(formatted_code, formatter)?;
    write!(formatted_code, "{}", Delimiter::Bracket.as_close_char())?;

    Ok(())
}

fn format_ptr(
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    ty: SquareBrackets<Box<Ty>>,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", PtrToken::AS_STR)?;
    write!(formatted_code, "{}", Delimiter::Bracket.as_open_char())?;
    ty.into_inner().format(formatted_code, formatter)?;
    write!(formatted_code, "{}", Delimiter::Bracket.as_close_char())?;

    Ok(())
}

fn format_slice(
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    slice_token: &Option<SliceToken>,
    ty: SquareBrackets<Box<Ty>>,
) -> Result<(), FormatterError> {
    if slice_token.is_some() {
        write!(formatted_code, "{}", SliceToken::AS_STR)?;
    }
    write!(formatted_code, "{}", Delimiter::Bracket.as_open_char())?;
    ty.into_inner().format(formatted_code, formatter)?;
    write!(formatted_code, "{}", Delimiter::Bracket.as_close_char())?;

    Ok(())
}

fn format_ref(
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
    mut_token: &Option<MutToken>,
    ty: &Ty,
) -> Result<(), FormatterError> {
    // TODO: Currently, the parser does not support declaring
    //       references on references without spaces between
    //       ampersands. E.g., `&&&T` is not supported and must
    //       be written as `& & &T`.
    //       See: https://github.com/FuelLabs/sway/issues/6808
    //       Until this issue is fixed, we need this workaround
    //       in case of referenced type `ty` being itself a
    //       reference.
    if !matches!(ty, Ty::Ref { .. }) {
        // TODO: Keep this code once the issue is fixed.
        write!(
            formatted_code,
            "{}{}",
            AmpersandToken::AS_STR,
            if mut_token.is_some() {
                format!("{} ", MutToken::AS_STR)
            } else {
                "".to_string()
            },
        )?;
        ty.format(formatted_code, formatter)?;
    } else {
        // TODO: This is the workaround if `ty` is a reference.
        write!(
            formatted_code,
            "{}{}{}",
            AmpersandToken::AS_STR,
            if mut_token.is_some() {
                format!("{} ", MutToken::AS_STR)
            } else {
                "".to_string()
            },
            // If we have the `mut`, we will also
            // get a space after it, so the next `&`
            // will be separated. Otherwise, insert space.
            if mut_token.is_some() {
                "".to_string()
            } else {
                " ".to_string()
            },
        )?;
        ty.format(formatted_code, formatter)?;
    }

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
            comma_token: _,
            tail,
        } = self
        {
            formatter.with_shape(
                formatter.shape.with_default_code_line(),
                |formatter| -> Result<(), FormatterError> {
                    head.format(formatted_code, formatter)?;
                    write!(formatted_code, "{} ", CommaToken::AS_STR)?;
                    tail.format(formatted_code, formatter)?;

                    Ok(())
                },
            )?;
        }

        Ok(())
    }
}

impl<T: LeafSpans + Clone> LeafSpans for Box<T> {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        (**self).leaf_spans()
    }
}

impl LeafSpans for Ty {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            Ty::Path(path) => path.leaf_spans(),
            Ty::Tuple(tuple) => tuple.leaf_spans(),
            Ty::Array(array) => array.leaf_spans(),
            Ty::StringSlice(str_token) => vec![ByteSpan::from(str_token.span())],
            Ty::StringArray { str_token, length } => {
                let mut collected_spans = vec![ByteSpan::from(str_token.span())];
                collected_spans.append(&mut length.leaf_spans());
                collected_spans
            }
            Ty::Infer { underscore_token } => vec![ByteSpan::from(underscore_token.span())],
            Ty::Ptr { ptr_token, ty } => {
                let mut collected_spans = vec![ByteSpan::from(ptr_token.span())];
                collected_spans.append(&mut ty.leaf_spans());
                collected_spans
            }
            Ty::Slice { slice_token, ty } => {
                let mut collected_spans = if let Some(slice_token) = slice_token {
                    vec![ByteSpan::from(slice_token.span())]
                } else {
                    vec![]
                };
                collected_spans.append(&mut ty.leaf_spans());
                collected_spans
            }
            Ty::Ref {
                ampersand_token,
                mut_token,
                ty,
            } => {
                let mut collected_spans = vec![ByteSpan::from(ampersand_token.span())];
                if let Some(mut_token) = mut_token {
                    collected_spans.push(ByteSpan::from(mut_token.span()));
                }
                collected_spans.append(&mut ty.leaf_spans());
                collected_spans
            }
            Ty::Never { bang_token } => vec![ByteSpan::from(bang_token.span())],
            Ty::Expr(expr) => expr.leaf_spans(),
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
