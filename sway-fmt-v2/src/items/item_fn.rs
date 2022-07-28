use crate::{
    config::items::ItemBraceStyle,
    fmt::{Format, FormattedCode, Formatter, FormatterError},
    utils::{
        bracket::{CurlyBrace, Parenthesis},
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, CodeBlockContents, FnArg, FnArgs, FnSignature, ItemFn};
use sway_types::Spanned;

impl Format for ItemFn {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.fn_signature.format(formatted_code, formatter)?;
        Self::open_curly_brace(formatted_code, formatter)?;
        self.body
            .clone()
            .into_inner()
            .format(formatted_code, formatter)?;
        Self::close_curly_brace(formatted_code, formatter)?;

        Ok(())
    }
}

impl CurlyBrace for ItemFn {
    fn open_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let brace_style = formatter.config.items.item_brace_style;
        let extra_width = formatter.config.whitespace.tab_spaces;
        let mut shape = formatter.shape;
        let open_brace = Delimiter::Brace.as_open_char();
        match brace_style {
            ItemBraceStyle::AlwaysNextLine => {
                // Add openning brace to the next line.
                writeln!(line, "\n{}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
            ItemBraceStyle::SameLineWhere => match shape.has_where_clause {
                true => {
                    writeln!(line, "{}", open_brace)?;
                    shape = shape.update_where_clause();
                    shape = shape.block_indent(extra_width);
                }
                false => {
                    writeln!(line, " {}", open_brace)?;
                    shape = shape.block_indent(extra_width);
                }
            },
            _ => {
                // TODO: implement PreferSameLine
                writeln!(line, " {}", open_brace)?;
                shape = shape.block_indent(extra_width);
            }
        }

        formatter.shape = shape;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // If shape is becoming left-most alligned or - indent just have the defualt shape
        formatter.shape.indent = formatter.shape.indent.block_unindent(formatter);
        writeln!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(formatter),
            Delimiter::Brace.as_close_char()
        )?;
        Ok(())
    }
}

impl Format for FnSignature {
    fn format(
        &self,
        formatted_code: &mut String,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}",
            formatter.shape.indent.to_string(formatter)
        )?;
        // `pub `
        if let Some(visibility_token) = &self.visibility {
            write!(formatted_code, "{} ", visibility_token.span().as_str())?;
        }
        // `fn ` + name
        write!(
            formatted_code,
            "{} {}",
            self.fn_token.span().as_str(),
            self.name.as_str()
        )?;
        // `<T>`
        if let Some(generics) = &self.generics.clone() {
            generics.format(formatted_code, formatter)?;
        }
        // `(`
        Self::open_parenthesis(formatted_code, formatter)?;
        // FnArgs
        match self.arguments.clone().into_inner() {
            FnArgs::Static(args) => {
                // TODO: Refactor into `Punctuated::format()`
                args.format(formatted_code, formatter)?;
            }
            FnArgs::NonStatic {
                self_token,
                mutable_self,
                args_opt,
            } => {
                // `mut `
                if let Some(mut_token) = mutable_self {
                    write!(formatted_code, "{} ", mut_token.span().as_str())?;
                }
                // `self`
                formatted_code.push_str(self_token.span().as_str());
                // `args_opt`
                if let Some(args) = args_opt {
                    // `, `
                    write!(formatted_code, "{} ", args.0.span().as_str())?;
                    // `Punctuated<FnArg, CommaToken>`
                    args.1.format(formatted_code, formatter)?;
                }
            }
        }
        // `)`
        Self::close_parenthesis(formatted_code, formatter)?;
        // `return_type_opt`
        if let Some(return_type) = &self.return_type_opt {
            write!(
                formatted_code,
                " {} ",
                return_type.0.span().as_str() // `->`
            )?;
            return_type.1.format(formatted_code, formatter)?; // `Ty`
        }
        // `WhereClause`
        if let Some(where_clause) = &self.where_clause_opt {
            where_clause.format(formatted_code, formatter)?;
            let mut shape = formatter.shape;
            shape = shape.update_where_clause();
            formatter.shape = shape;
        }
        Ok(())
    }
}

// We will need to add logic to handle the case of long fn arguments, and break into new line
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

impl Format for CodeBlockContents {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        for statement in self.statements.iter() {
            statement.format(formatted_code, formatter)?;
        }
        if let Some(final_expr) = &self.final_expr_opt {
            write!(
                formatted_code,
                "{}",
                formatter.shape.indent.to_string(formatter)
            )?;
            final_expr.format(formatted_code, formatter)?;
            writeln!(formatted_code)?;
        }

        Ok(())
    }
}

// TODO: Use this in `Punctuated::format()`
impl Format for FnArg {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.pattern.format(formatted_code, formatter)?;
        // `: `
        write!(formatted_code, "{} ", self.colon_token.span().as_str())?;
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

impl LeafSpans for CodeBlockContents {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_span = Vec::new();
        for statement in self.statements.iter() {
            collected_span.append(&mut statement.leaf_spans());
        }
        if let Some(expr) = &self.final_expr_opt {
            collected_span.append(&mut expr.leaf_spans());
        }
        collected_span
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
        // TODO add where, I will add where for all items at once.
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
                mutable_self,
                args_opt,
            } => {
                collected_spans.push(ByteSpan::from(self_token.span()));
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
