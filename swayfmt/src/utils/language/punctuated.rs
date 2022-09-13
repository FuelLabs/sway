use crate::{
    constants::RAW_MODIFIER,
    formatter::{shape::LineStyle, *},
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    keywords::CommaToken, punctuated::Punctuated, token::PunctKind, StorageField, TypeField,
};
use sway_types::{Ident, Spanned};

impl<T, P> Format for Punctuated<T, P>
where
    T: Format,
    P: Format,
{
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        if !self.value_separator_pairs.is_empty() || self.final_value_opt.is_some() {
            match formatter.shape.code_line.line_style {
                LineStyle::Normal => {
                    let value_pairs = &self.value_separator_pairs;
                    for (type_field, punctuation) in value_pairs.iter() {
                        type_field.format(formatted_code, formatter)?;
                        punctuation.format(formatted_code, formatter)?;
                        write!(formatted_code, " ")?;
                    }

                    if let Some(final_value) = &self.final_value_opt {
                        final_value.format(formatted_code, formatter)?;
                    }
                }
                LineStyle::Inline => {
                    write!(formatted_code, " ")?;
                    let mut value_pairs_iter = self.value_separator_pairs.iter().peekable();
                    for (type_field, punctuation) in value_pairs_iter.clone() {
                        type_field.format(formatted_code, formatter)?;
                        punctuation.format(formatted_code, formatter)?;

                        if value_pairs_iter.peek().is_some() {
                            write!(formatted_code, " ")?;
                        }
                    }
                    if let Some(final_value) = &self.final_value_opt {
                        final_value.format(formatted_code, formatter)?;
                    } else {
                        formatted_code.pop();
                        formatted_code.pop();
                    }
                    write!(formatted_code, " ")?;
                }
                LineStyle::Multiline => {
                    writeln!(formatted_code)?;
                    let mut value_pairs_iter = self.value_separator_pairs.iter().peekable();
                    for (type_field, comma_token) in value_pairs_iter.clone() {
                        write!(
                            formatted_code,
                            "{}",
                            &formatter.shape.indent.to_string(&formatter.config)?
                        )?;
                        type_field.format(formatted_code, formatter)?;

                        if value_pairs_iter.peek().is_some() {
                            comma_token.format(formatted_code, formatter)?;
                            writeln!(formatted_code)?;
                        }
                    }
                    if let Some(final_value) = &self.final_value_opt {
                        write!(
                            formatted_code,
                            "{}",
                            &formatter.shape.indent.to_string(&formatter.config)?
                        )?;
                        final_value.format(formatted_code, formatter)?;
                        writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<T, P> LeafSpans for Punctuated<T, P>
where
    T: LeafSpans + Clone,
    P: LeafSpans + Clone,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let value_pairs = &self.value_separator_pairs;
        for pair in value_pairs.iter() {
            let p_comment_spans = pair.1.leaf_spans();
            // Since we do not want to have comments between T and P we are extending the ByteSpans coming from T with spans coming from P
            // Since formatter can insert a trailing comma after a field, comments next to a field can be falsely inserted between the comma and the field
            // So we shouldn't allow inserting comments (or searching for one) between T and P as in Punctuated scenerio this can/will result in formattings that breaks the build process
            let mut comment_spans = pair
                .0
                .leaf_spans()
                .iter_mut()
                .map(|comment_map| {
                    // Since the length of P' ByteSpan is same for each pair we are using the first one's length for all of the pairs.
                    // This assumtion always holds because for each pair P is formatted to same str so the length is going to be the same.
                    // For example when P is CommaToken, the length of P is always 1.
                    comment_map.end += p_comment_spans[0].len();
                    comment_map.clone()
                })
                .collect();
            collected_spans.append(&mut comment_spans)
        }
        if let Some(final_value) = &self.final_value_opt {
            collected_spans.append(&mut final_value.leaf_spans());
        }
        collected_spans
    }
}

impl Format for Ident {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self.is_raw_ident() {
            true => write!(formatted_code, "{}{}", RAW_MODIFIER, self.span().as_str())?,
            false => write!(formatted_code, "{}", self.span().as_str())?,
        }

        Ok(())
    }
}

impl Format for TypeField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{} ",
            self.name.span().as_str(),
            self.colon_token.span().as_str(),
        )?;
        self.ty.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl Format for StorageField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let prev_state = formatter.shape.code_line;
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Normal);
        write!(
            formatted_code,
            "{}{} ",
            self.name.span().as_str(),
            self.colon_token.span().as_str(),
        )?;
        self.ty.format(formatted_code, formatter)?;
        write!(formatted_code, " {} ", self.eq_token.span().as_str())?;

        formatter.shape.update_line_settings(prev_state);
        self.initializer.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl Format for CommaToken {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span().as_str())?;

        Ok(())
    }
}
