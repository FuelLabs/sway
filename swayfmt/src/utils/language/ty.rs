use crate::{
    formatter::*,
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    brackets::SquareBrackets,
    expr::Expr,
    keywords::{AmpersandToken, MutToken, PtrToken, SliceToken, StrToken, Token, UnderscoreToken},
    ty::{Ty, TyArrayDescriptor, TyTupleDescriptor},
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
            Self::Infer { underscore_token } => format_infer(formatted_code, underscore_token),
            Self::Path(path_ty) => path_ty.format(formatted_code, formatter),
            Self::StringSlice(_) => {
                write!(formatted_code, "str")?;
                Ok(())
            }
            Self::StringArray { str_token, length } => {
                format_str(formatted_code, str_token.clone(), length.clone())
            }
            Self::Tuple(tup_descriptor) => {
                write!(formatted_code, "{}", Delimiter::Parenthesis.as_open_char())?;
                tup_descriptor.get().format(formatted_code, formatter)?;
                write!(formatted_code, "{}", Delimiter::Parenthesis.as_close_char())?;
                Ok(())
            }
            Self::Ptr { ptr_token, ty } => {
                format_ptr(formatted_code, ptr_token.clone(), ty.clone())
            }
            Self::Slice { slice_token, ty } => {
                format_slice(formatted_code, slice_token.clone(), ty.clone())
            }
            Self::Ref {
                ampersand_token,
                mut_token,
                ty,
            } => format_ref(
                formatted_code,
                ampersand_token.clone(),
                mut_token.clone(),
                ty.clone(),
            ),
            Self::Never { bang_token } => {
                write!(formatted_code, "{}", bang_token.span().as_str(),)?;
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

fn format_ptr(
    formatted_code: &mut FormattedCode,
    ptr_token: PtrToken,
    ty: SquareBrackets<Box<Ty>>,
) -> Result<(), FormatterError> {
    write!(
        formatted_code,
        "{}[{}]",
        ptr_token.span().as_str(),
        ty.into_inner().span().as_str()
    )?;
    Ok(())
}

fn format_slice(
    formatted_code: &mut FormattedCode,
    slice_token: SliceToken,
    ty: SquareBrackets<Box<Ty>>,
) -> Result<(), FormatterError> {
    write!(
        formatted_code,
        "{}[{}]",
        slice_token.span().as_str(),
        ty.into_inner().span().as_str()
    )?;
    Ok(())
}

fn format_ref(
    formatted_code: &mut FormattedCode,
    ampersand_token: AmpersandToken,
    mut_token: Option<MutToken>,
    ty: Box<Ty>,
) -> Result<(), FormatterError> {
    write!(
        formatted_code,
        "{}{}{}",
        ampersand_token.span().as_str(),
        if let Some(mut_token) = mut_token {
            format!("{} ", mut_token.span().as_str())
        } else {
            "".to_string()
        },
        ty.span().as_str()
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
            formatter.with_shape(
                formatter.shape.with_default_code_line(),
                |formatter| -> Result<(), FormatterError> {
                    head.format(formatted_code, formatter)?;
                    write!(formatted_code, "{} ", comma_token.ident().as_str())?;
                    tail.format(formatted_code, formatter)?;

                    Ok(())
                },
            )?;
        }

        Ok(())
    }
}

impl LeafSpans for Box<Ty> {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        self.as_ref().leaf_spans()
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
                let mut collected_spans = vec![ByteSpan::from(slice_token.span())];
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
