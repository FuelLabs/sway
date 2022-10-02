use crate::{
    config::items::ItemBraceStyle,
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {CurlyBrace, Parenthesis},
    },
};
use std::fmt::Write;
use sway_ast::{
    token::Delimiter, Braces, CommaToken, PathExpr, Pattern, PatternStructField, Punctuated,
};
use sway_types::Spanned;

impl Format for Pattern {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Wildcard { underscore_token } => {
                formatted_code.push_str(underscore_token.span().as_str())
            }
            Self::Var {
                reference,
                mutable,
                name,
            } => {
                if let Some(ref_token) = reference {
                    write!(formatted_code, "{} ", ref_token.span().as_str())?;
                }
                if let Some(mut_token) = mutable {
                    write!(formatted_code, "{} ", mut_token.span().as_str())?;
                }
                name.format(formatted_code, formatter)?;
            }
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::Constant(path) => path.format(formatted_code, formatter)?,
            Self::Constructor { path, args } => {
                // TODO: add a check for width of whether to be normal or multiline
                formatter.with_shape(
                    formatter.shape.with_default_code_line(),
                    |formatter| -> Result<(), FormatterError> {
                        path.format(formatted_code, formatter)?;
                        Self::open_parenthesis(formatted_code, formatter)?;
                        args.get().format(formatted_code, formatter)?;
                        Self::close_parenthesis(formatted_code, formatter)?;
                        Ok(())
                    },
                )?;
            }
            Self::Struct { path, fields } => {
                formatter.with_shape(
                    formatter
                        .shape
                        .with_code_line_from(LineStyle::default(), ExprKind::Struct),
                    |formatter| -> Result<(), FormatterError> {
                        // get the length in chars of the code_line in a single line format,
                        // this include the path
                        let mut buf = FormattedCode::new();
                        let mut temp_formatter = Formatter::default();
                        temp_formatter
                            .shape
                            .code_line
                            .update_line_style(LineStyle::Inline);
                        format_pattern_struct(path, fields, &mut buf, &mut temp_formatter)?;

                        // get the largest field size and the size of the body
                        let (field_width, body_width) =
                            get_field_width(fields.get(), &mut formatter.clone())?;

                        // changes to the actual formatter
                        let expr_width = buf.chars().count() as usize;
                        formatter.shape.code_line.add_width(expr_width);
                        formatter.shape.get_line_style(
                            Some(field_width),
                            Some(body_width),
                            &formatter.config,
                        );

                        format_pattern_struct(path, fields, formatted_code, formatter)?;

                        Ok(())
                    },
                )?;
            }
            Self::Tuple(args) => {
                Self::open_parenthesis(formatted_code, formatter)?;
                args.get().format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
        }
        Ok(())
    }
}

// Currently these just push their respective chars, we may need to change this
impl Parenthesis for Pattern {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_close_char())?;
        Ok(())
    }
}
impl CurlyBrace for Pattern {
    fn open_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                write!(line, "\n{}", Delimiter::Brace.as_open_char())?;
                formatter.shape.block_indent(&formatter.config);
            }
            _ => {
                // Add opening brace to the same line
                write!(line, " {}", Delimiter::Brace.as_open_char())?;
                formatter.shape.block_indent(&formatter.config);
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Unindent by one block
        formatter.shape.block_unindent(&formatter.config);
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => write!(line, "{}", Delimiter::Brace.as_close_char())?,
            _ => write!(
                line,
                "{}{}",
                formatter.shape.indent.to_string(&formatter.config)?,
                Delimiter::Brace.as_close_char()
            )?,
        }

        Ok(())
    }
}

impl Format for PatternStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Rest { token } => {
                write!(formatted_code, "{}", token.span().as_str())?;
            }
            Self::Field {
                field_name,
                pattern_opt,
            } => {
                write!(formatted_code, "{}", field_name.span().as_str())?;
                if let Some((colon_token, pattern)) = pattern_opt {
                    write!(formatted_code, "{}", colon_token.span().as_str())?;
                    pattern.format(formatted_code, formatter)?;
                }
            }
        }

        Ok(())
    }
}

fn get_field_width(
    fields: &Punctuated<PatternStructField, CommaToken>,
    formatter: &mut Formatter,
) -> Result<(usize, usize), FormatterError> {
    let mut largest_field: usize = 0;
    let mut body_width: usize = 3; // this is taking into account the opening brace, the following space and the ending brace.
    for (field, comma_token) in &fields.value_separator_pairs {
        let mut field_str = FormattedCode::new();
        field.format(&mut field_str, formatter)?;
        let mut field_length = field_str.chars().count() as usize;

        field_length += comma_token.span().as_str().chars().count() as usize;
        body_width += &field_length + 1; // accounting for the following space

        if field_length > largest_field {
            largest_field = field_length;
        }
    }
    if let Some(final_value) = &fields.final_value_opt {
        let mut field_str = FormattedCode::new();
        final_value.format(&mut field_str, formatter)?;
        let field_length = field_str.chars().count() as usize;

        body_width += &field_length + 1; // accounting for the following space

        if field_length > largest_field {
            largest_field = field_length;
        }
    }

    Ok((largest_field, body_width))
}
fn format_pattern_struct(
    path: &PathExpr,
    fields: &Braces<Punctuated<PatternStructField, CommaToken>>,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    path.format(formatted_code, formatter)?;
    Pattern::open_curly_brace(formatted_code, formatter)?;
    let fields = &fields.get();
    match formatter.shape.code_line.line_style {
        LineStyle::Inline => fields.format(formatted_code, formatter)?,
        // TODO: add field alignment
        _ => fields.format(formatted_code, formatter)?,
    }
    Pattern::close_curly_brace(formatted_code, formatter)?;

    Ok(())
}

impl LeafSpans for Pattern {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            Pattern::Wildcard { underscore_token } => {
                collected_spans.push(ByteSpan::from(underscore_token.span()));
            }
            Pattern::Var {
                reference,
                mutable,
                name,
            } => {
                if let Some(reference) = reference {
                    collected_spans.push(ByteSpan::from(reference.span()));
                }
                if let Some(mutable) = mutable {
                    collected_spans.push(ByteSpan::from(mutable.span()));
                }
                collected_spans.push(ByteSpan::from(name.span()));
            }
            Pattern::Literal(literal) => {
                collected_spans.append(&mut literal.leaf_spans());
            }
            Pattern::Constant(constant) => {
                collected_spans.append(&mut constant.leaf_spans());
            }
            Pattern::Constructor { path, args } => {
                collected_spans.append(&mut path.leaf_spans());
                collected_spans.append(&mut args.leaf_spans());
            }
            Pattern::Struct { path, fields } => {
                collected_spans.append(&mut path.leaf_spans());
                collected_spans.append(&mut fields.leaf_spans());
            }
            Pattern::Tuple(tuple) => {
                collected_spans.append(&mut tuple.leaf_spans());
            }
        }
        collected_spans
    }
}

impl LeafSpans for PatternStructField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            PatternStructField::Rest { token } => {
                collected_spans.push(ByteSpan::from(token.span()));
            }
            PatternStructField::Field {
                field_name,
                pattern_opt,
            } => {
                collected_spans.push(ByteSpan::from(field_name.span()));
                if let Some(pattern) = pattern_opt {
                    collected_spans.push(ByteSpan::from(pattern.0.span()));
                    collected_spans.append(&mut pattern.1.leaf_spans());
                }
            }
        }
        collected_spans
    }
}
