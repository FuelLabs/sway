use crate::{
    constants::RAW_MODIFIER,
    formatter::{shape::LineStyle, *},
    utils::map::byte_span::{ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    keywords::CommaToken, punctuated::Punctuated, ConfigurableField, StorageField, TypeField,
};
use sway_types::{ast::PunctKind, Ident, Spanned};

use super::expr::should_write_multiline;

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
                    write!(
                        formatted_code,
                        "{}",
                        format_generic_pair(
                            &self.value_separator_pairs,
                            &self.final_value_opt,
                            formatter
                        )?
                    )?;
                }
                LineStyle::Inline => {
                    write!(
                        formatted_code,
                        " {} ",
                        format_generic_pair(
                            &self.value_separator_pairs,
                            &self.final_value_opt,
                            formatter
                        )?
                    )?;
                }
                LineStyle::Multiline => {
                    if !formatted_code.ends_with('\n') {
                        writeln!(formatted_code)?;
                    }
                    if !self.value_separator_pairs.is_empty() || self.final_value_opt.is_some() {
                        formatter.write_indent_into_buffer(formatted_code)?;
                    }

                    let mut is_value_too_long = false;
                    let value_separator_pairs = formatter.with_shape(
                        formatter.shape.with_default_code_line(),
                        |formatter| -> Result<Vec<(String, String)>, FormatterError> {
                            self.value_separator_pairs
                                .iter()
                                .map(|(type_field, comma_token)| {
                                    let mut field = FormattedCode::new();
                                    let mut comma = FormattedCode::new();
                                    type_field.format(&mut field, formatter)?;
                                    comma_token.format(&mut comma, formatter)?;
                                    if field.len()
                                        > formatter.shape.width_heuristics.short_array_element_width
                                    {
                                        is_value_too_long = true;
                                    }
                                    Ok((
                                        field.trim_start().to_owned(),
                                        comma.trim_start().to_owned(),
                                    ))
                                })
                                .collect()
                        },
                    )?;

                    let mut iter = value_separator_pairs.iter().peekable();

                    while let Some((type_field, comma_token)) = iter.next() {
                        write!(formatted_code, "{}{}", type_field, comma_token)?;
                        if iter.peek().is_none() && self.final_value_opt.is_none() {
                            break;
                        }
                        if is_value_too_long || should_write_multiline(formatted_code, formatter) {
                            writeln!(formatted_code)?;
                            formatter.write_indent_into_buffer(formatted_code)?;
                        } else {
                            write!(formatted_code, " ")?;
                        }
                    }
                    if let Some(final_value) = &self.final_value_opt {
                        final_value.format(formatted_code, formatter)?;
                        write!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                    }
                    if !formatted_code.ends_with('\n') {
                        writeln!(formatted_code)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn format_generic_pair<T, P>(
    value_separator_pairs: &[(T, P)],
    final_value_opt: &Option<Box<T>>,
    formatter: &mut Formatter,
) -> Result<FormattedCode, FormatterError>
where
    T: Format,
    P: Format,
{
    let len = value_separator_pairs.len();
    let mut ts: Vec<String> = Vec::with_capacity(len);
    let mut ps: Vec<String> = Vec::with_capacity(len);
    for (t, p) in value_separator_pairs.iter() {
        let mut t_buf = FormattedCode::new();
        t.format(&mut t_buf, formatter)?;
        ts.push(t_buf);

        let mut p_buf = FormattedCode::new();
        p.format(&mut p_buf, formatter)?;
        ps.push(p_buf);
    }
    if let Some(final_value) = final_value_opt {
        let mut buf = FormattedCode::new();
        final_value.format(&mut buf, formatter)?;
        ts.push(buf);
    } else {
        // reduce the number of punct by 1
        // this is safe since the number of
        // separator pairs is always equal
        ps.truncate(ts.len() - 1);
    }
    for (t, p) in ts.iter_mut().zip(ps.iter()) {
        write!(t, "{p}")?;
    }
    Ok(ts.join(" "))
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
            // So we shouldn't allow inserting comments (or searching for one) between T and P as in Punctuated scenario this can/will result in formatting that breaks the build process
            let mut comment_spans = pair
                .0
                .leaf_spans()
                .iter_mut()
                .map(|comment_map| {
                    // Since the length of P' ByteSpan is same for each pair we are using the first one's length for all of the pairs.
                    // This assumption always holds because for each pair P is formatted to same str so the length is going to be the same.
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
        // If there is a visibility token add it to the formatted_code with a ` ` after it.
        if let Some(visibility) = &self.visibility {
            write!(formatted_code, "{} ", visibility.span().as_str())?;
        }
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

impl Format for ConfigurableField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.with_shape(
            formatter.shape.with_default_code_line(),
            |formatter| -> Result<(), FormatterError> {
                write!(
                    formatted_code,
                    "{}{} ",
                    self.name.span().as_str(),
                    self.colon_token.span().as_str(),
                )?;
                self.ty.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", self.eq_token.span().as_str())?;

                Ok(())
            },
        )?;

        self.initializer.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl Format for StorageField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.with_shape(
            formatter.shape.with_default_code_line(),
            |formatter| -> Result<(), FormatterError> {
                write!(
                    formatted_code,
                    "{}{} ",
                    self.name.span().as_str(),
                    self.colon_token.span().as_str(),
                )?;
                self.ty.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", self.eq_token.span().as_str())?;

                Ok(())
            },
        )?;

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
