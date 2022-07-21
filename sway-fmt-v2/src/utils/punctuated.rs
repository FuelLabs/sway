use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::comments::{ByteSpan, CommentVisitor},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{keywords::CommaToken, punctuated::Punctuated, StorageField, TypeField};
use sway_types::{Ident, Spanned};

impl<T, P> CommentVisitor for Punctuated<T, P>
where
    T: CommentVisitor + Clone,
    P: CommentVisitor + Clone,
{
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let value_pairs = &self.value_separator_pairs;
        for pair in value_pairs.iter() {
            let p_comment_spans = pair.1.collect_spans();
            // Since we do not want to have comments between T and P we are extending the ByteSpans coming from T with spans coming from P
            // Since formatter can insert a trailing comma after a field, comments next to a field can be falsely inserted between the comma and the field
            // So we shouldn't allow inserting comments (or searching for one) between T and P as in Punctuated scenerio this can/will result in formattings that breaks the build process
            let mut comment_spans = pair
                .0
                .collect_spans()
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
            collected_spans.append(&mut final_value.collect_spans());
        }
        collected_spans
    }
}

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
        // format and add Type & Punct
        let value_pairs = &self.value_separator_pairs;
        for pair in value_pairs.iter() {
            pair.0.format(formatted_code, formatter)?;
            pair.1.format(formatted_code, formatter)?;
        }

        // add final value, if any
        if let Some(final_value) = &self.final_value_opt {
            final_value.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl Format for Ident {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span().as_str())?;
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
        write!(
            formatted_code,
            "{}{} ",
            self.name.span().as_str(),
            self.colon_token.span().as_str(),
        )?;
        self.ty.format(formatted_code, formatter)?;
        write!(formatted_code, " {} ", self.eq_token.span().as_str())?;
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
        write!(formatted_code, "{} ", self.span().as_str())?;
        Ok(())
    }
}
