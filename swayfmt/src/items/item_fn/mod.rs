use crate::{
    comments::{rewrite_with_comments, write_comments},
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
    keywords::{MutToken, RefToken, SelfToken, Token},
    FnArg, FnArgs, FnSignature, ItemFn,
};
use sway_types::{ast::Delimiter, Spanned};

#[cfg(test)]
mod tests;

impl Format for ItemFn {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // Required for comment formatting
        let start_len = formatted_code.len();

        formatter.with_shape(
            formatter
                .shape
                .with_code_line_from(LineStyle::Normal, ExprKind::Function),
            |formatter| -> Result<(), FormatterError> {
                self.fn_signature.format(formatted_code, formatter)?;
                let body = self.body.get();
                if !body.statements.is_empty() || body.final_expr_opt.is_some() {
                    Self::open_curly_brace(formatted_code, formatter)?;
                    formatter.indent();
                    body.format(formatted_code, formatter)?;

                    if let Some(final_expr_opt) = body.final_expr_opt.as_ref() {
                        write_comments(
                            formatted_code,
                            final_expr_opt.span().end()..self.span().end(),
                            formatter,
                        )?;
                    }

                    Self::close_curly_brace(formatted_code, formatter)?;
                } else {
                    Self::open_curly_brace(formatted_code, formatter)?;
                    formatter.indent();
                    let comments = write_comments(formatted_code, self.span().into(), formatter)?;
                    if !comments {
                        formatter.unindent();
                    }
                    Self::close_curly_brace(formatted_code, formatter)?;
                }

                rewrite_with_comments::<ItemFn>(
                    formatter,
                    self.span(),
                    self.leaf_spans(),
                    formatted_code,
                    start_len,
                )?;

                Ok(())
            },
        )?;

        Ok(())
    }
}

impl CurlyBrace for ItemFn {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add opening brace to the next line.
                writeln!(line, "\n{open_brace}")?;
            }
            ItemBraceStyle::SameLineWhere => match formatter.shape.code_line.has_where_clause {
                true => {
                    write!(line, "{open_brace}")?;
                    formatter.shape.code_line.update_where_clause(false);
                }
                false => {
                    write!(line, " {open_brace}")?;
                }
            },
            _ => {
                // TODO: implement PreferSameLine
                writeln!(line, " {open_brace}")?;
            }
        }

        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.unindent();
        write!(
            line,
            "{}{}",
            formatter.indent_to_str()?,
            Delimiter::Brace.as_close_char()
        )?;

        Ok(())
    }
}

impl Format for FnSignature {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        formatter.shape.code_line.has_where_clause = formatter.with_shape(
            formatter.shape,
            |formatter| -> Result<bool, FormatterError> {
                let mut fn_sig = FormattedCode::new();
                let mut fn_args = FormattedCode::new();
                let mut temp_formatter = Formatter::default();
                format_fn_sig(self, &mut fn_sig, &mut temp_formatter)?;
                format_fn_args(self.arguments.get(), &mut fn_args, &mut temp_formatter)?;

                let fn_sig_width = fn_sig.chars().count() + 2; // add two for opening brace + space
                let fn_args_width = fn_args.chars().count();

                formatter.shape.code_line.update_width(fn_sig_width);
                formatter
                    .shape
                    .get_line_style(None, Some(fn_args_width), &formatter.config);

                format_fn_sig(self, formatted_code, formatter)?;

                Ok(formatter.shape.code_line.has_where_clause)
            },
        )?;

        Ok(())
    }
}

fn format_fn_sig(
    fn_sig: &FnSignature,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    // `pub `
    if let Some(visibility_token) = &fn_sig.visibility {
        write!(formatted_code, "{} ", visibility_token.span().as_str())?;
    }
    // `fn ` + name
    write!(formatted_code, "{} ", fn_sig.fn_token.span().as_str())?;
    fn_sig.name.format(formatted_code, formatter)?;
    // `<T>`
    if let Some(generics) = &fn_sig.generics {
        generics.format(formatted_code, formatter)?;
    }
    // `(`
    FnSignature::open_parenthesis(formatted_code, formatter)?;
    // FnArgs
    format_fn_args(fn_sig.arguments.get(), formatted_code, formatter)?;
    // `)`
    FnSignature::close_parenthesis(formatted_code, formatter)?;
    // `return_type_opt`
    if let Some((right_arrow, ty)) = &fn_sig.return_type_opt {
        write!(
            formatted_code,
            " {} ",
            right_arrow.ident().as_str() // `->`
        )?;
        ty.format(formatted_code, formatter)?; // `Ty`
    }
    // `WhereClause`
    if let Some(where_clause) = &fn_sig.where_clause_opt {
        writeln!(formatted_code)?;
        where_clause.format(formatted_code, formatter)?;
        formatter.shape.code_line.update_where_clause(true);
    }

    Ok(())
}

fn format_fn_args(
    fn_args: &FnArgs,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    match fn_args {
        FnArgs::Static(args) => match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                if !args.value_separator_pairs.is_empty() || args.final_value_opt.is_some() {
                    formatter.indent();
                    args.format(formatted_code, formatter)?;
                    formatter.unindent();
                    write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                }
            }
            _ => args.format(formatted_code, formatter)?,
        },
        FnArgs::NonStatic {
            self_token,
            ref_self,
            mutable_self,
            args_opt,
        } => {
            match formatter.shape.code_line.line_style {
                LineStyle::Multiline => {
                    formatter.indent();
                    write!(formatted_code, "\n{}", formatter.indent_to_str()?)?;
                    format_self(self_token, ref_self, mutable_self, formatted_code)?;
                    // `args_opt`
                    if let Some((comma, args)) = args_opt {
                        // `, `
                        write!(formatted_code, "{}", comma.ident().as_str())?;
                        // `Punctuated<FnArg, CommaToken>`
                        args.format(formatted_code, formatter)?;
                    }
                }
                _ => {
                    format_self(self_token, ref_self, mutable_self, formatted_code)?;
                    // `args_opt`
                    if let Some((comma, args)) = args_opt {
                        // `, `
                        write!(formatted_code, "{} ", comma.ident().as_str())?;
                        // `Punctuated<FnArg, CommaToken>`
                        args.format(formatted_code, formatter)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn format_self(
    self_token: &SelfToken,
    ref_self: &Option<RefToken>,
    mutable_self: &Option<MutToken>,
    formatted_code: &mut FormattedCode,
) -> Result<(), FormatterError> {
    // `ref `
    if let Some(ref_token) = ref_self {
        write!(formatted_code, "{} ", ref_token.span().as_str())?;
    }
    // `mut `
    if let Some(mut_token) = mutable_self {
        write!(formatted_code, "{} ", mut_token.span().as_str())?;
    }
    // `self`
    write!(formatted_code, "{}", self_token.span().as_str())?;

    Ok(())
}

impl Parenthesis for FnSignature {
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

impl Format for FnArg {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.pattern.format(formatted_code, formatter)?;
        // `: `
        write!(formatted_code, "{} ", self.colon_token.span().as_str())?;

        write_comments(
            formatted_code,
            self.colon_token.span().end()..self.ty.span().start(),
            formatter,
        )?;
        // `Ty`
        self.ty.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl LeafSpans for ItemFn {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.append(&mut self.fn_signature.leaf_spans());
        collected_spans.append(&mut self.body.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for FnSignature {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from(visibility.span()));
        }
        collected_spans.push(ByteSpan::from(self.fn_token.span()));
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(generics) = &self.generics {
            collected_spans.push(ByteSpan::from(generics.parameters.span()));
        }
        collected_spans.append(&mut self.arguments.leaf_spans());
        if let Some(return_type) = &self.return_type_opt {
            collected_spans.append(&mut return_type.leaf_spans());
        }
        if let Some(where_clause) = &self.where_clause_opt {
            collected_spans.append(&mut where_clause.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for FnArgs {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match &self {
            FnArgs::Static(arg_static) => {
                collected_spans.append(&mut arg_static.leaf_spans());
            }
            FnArgs::NonStatic {
                self_token,
                ref_self,
                mutable_self,
                args_opt,
            } => {
                collected_spans.push(ByteSpan::from(self_token.span()));
                if let Some(reference) = ref_self {
                    collected_spans.push(ByteSpan::from(reference.span()));
                }
                if let Some(mutable) = mutable_self {
                    collected_spans.push(ByteSpan::from(mutable.span()));
                }
                if let Some(args) = args_opt {
                    collected_spans.append(&mut args.leaf_spans());
                }
            }
        };
        collected_spans
    }
}

impl LeafSpans for FnArg {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.append(&mut self.pattern.leaf_spans());
        collected_spans.push(ByteSpan::from(self.colon_token.span()));
        collected_spans.push(ByteSpan::from(self.ty.span()));
        collected_spans
    }
}
