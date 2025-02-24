use crate::{
    formatter::{
        shape::{ExprKind, LineStyle},
        *,
    },
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {CurlyBrace, Parenthesis, SquareBracket},
    },
};
use std::fmt::Write;
use sway_ast::{
    brackets::Parens, keywords::*, punctuated::Punctuated, Braces, CodeBlockContents, Expr,
    ExprStructField, IfExpr, MatchBranch, PathExpr, PathExprSegment,
};
use sway_types::{ast::Delimiter, Spanned};

pub(crate) mod abi_cast;
pub(crate) mod asm_block;
pub(crate) mod assignable;
pub(crate) mod code_block;
pub(crate) mod collections;
pub(crate) mod conditional;
pub(crate) mod struct_field;

#[cfg(test)]
mod tests;

#[inline]
fn two_parts_expr(
    lhs: &Expr,
    operator: &str,
    rhs: &Expr,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    let mut rhs_code = FormattedCode::new();
    rhs.format(&mut rhs_code, formatter)?;

    if !formatter.shape.code_line.expr_new_line
        && rhs_code.len() > formatter.shape.width_heuristics.collection_width
    {
        // Right hand side is too long to fit in a single line, and
        // the current expr is not being rendered multiline at the
        // expr level, then add an indentation to the following
        // expression and generate the code
        formatter.with_shape(
            formatter
                .shape
                .with_code_line_from(LineStyle::Multiline, ExprKind::Undetermined),
            |formatter| -> Result<(), FormatterError> {
                formatter.shape.code_line.update_expr_new_line(true);

                lhs.format(formatted_code, formatter)?;
                formatter.indent();
                write!(
                    formatted_code,
                    "\n{}{} ",
                    formatter.indent_to_str()?,
                    operator,
                )?;
                rhs.format(formatted_code, formatter)?;
                formatter.unindent();
                Ok(())
            },
        )?;
    } else {
        lhs.format(formatted_code, formatter)?;
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                write!(
                    formatted_code,
                    "\n{}{} ",
                    formatter.indent_to_str()?,
                    operator,
                )?;
            }
            _ => {
                write!(formatted_code, " {} ", operator)?;
            }
        }
        write!(formatted_code, "{}", rhs_code)?;
    }
    Ok(())
}

impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Error(_, _) => {
                return Err(FormatterError::SyntaxError);
            }
            Self::Path(path) => path.format(formatted_code, formatter)?,
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::AbiCast { abi_token: _, args } => {
                write!(formatted_code, "{}", AbiToken::AS_STR)?;
                args.get().format(formatted_code, formatter)?;
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
                        format_expr_struct(path, fields, &mut buf, &mut temp_formatter)?;

                        // get the largest field size and the size of the body
                        let (field_width, body_width) =
                            get_field_width(fields.get(), &mut formatter.clone())?;

                        formatter.shape.code_line.update_expr_new_line(true);

                        // changes to the actual formatter
                        let expr_width = buf.chars().count();
                        formatter.shape.code_line.update_width(expr_width);
                        formatter.shape.get_line_style(
                            Some(field_width),
                            Some(body_width),
                            &formatter.config,
                        );

                        format_expr_struct(path, fields, formatted_code, formatter)?;

                        Ok(())
                    },
                )?;
            }
            Self::Tuple(tuple_descriptor) => {
                formatter.with_shape(
                    formatter
                        .shape
                        .with_code_line_from(LineStyle::default(), ExprKind::Collection),
                    |formatter| -> Result<(), FormatterError> {
                        // get the length in chars of the code_line in a normal line format
                        let mut buf = FormattedCode::new();
                        let mut temp_formatter = Formatter::default();
                        let tuple_descriptor = tuple_descriptor.get();
                        tuple_descriptor.format(&mut buf, &mut temp_formatter)?;
                        let body_width = buf.chars().count();

                        formatter.shape.code_line.update_width(body_width);
                        formatter
                            .shape
                            .get_line_style(None, Some(body_width), &formatter.config);

                        tuple_descriptor.format(formatted_code, formatter)?;

                        Ok(())
                    },
                )?;
            }
            Self::Parens(expr) => {
                if formatter.shape.code_line.expr_new_line {
                    formatter.indent();
                }
                Self::open_parenthesis(formatted_code, formatter)?;
                expr.get().format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
                if formatter.shape.code_line.expr_new_line {
                    formatter.unindent();
                }
            }
            Self::Block(code_block) => {
                if !code_block.get().statements.is_empty()
                    || code_block.get().final_expr_opt.is_some()
                {
                    CodeBlockContents::open_curly_brace(formatted_code, formatter)?;
                    code_block.get().format(formatted_code, formatter)?;
                    CodeBlockContents::close_curly_brace(formatted_code, formatter)?;
                } else {
                    write!(formatted_code, "{{}}")?;
                }
            }
            Self::Array(array_descriptor) => {
                formatter.with_shape(
                    formatter
                        .shape
                        .with_code_line_from(LineStyle::default(), ExprKind::Collection),
                    |formatter| -> Result<(), FormatterError> {
                        // get the length in chars of the code_line in a normal line format
                        let mut buf = FormattedCode::new();
                        let mut temp_formatter = Formatter::default();
                        let array_descriptor = array_descriptor.get();
                        array_descriptor.format(&mut buf, &mut temp_formatter)?;
                        let body_width = buf.chars().count();

                        formatter.shape.code_line.add_width(body_width);
                        formatter
                            .shape
                            .get_line_style(None, Some(body_width), &formatter.config);

                        if formatter.shape.code_line.line_style == LineStyle::Multiline {
                            // Expr needs to be split into multiple lines
                            array_descriptor.format(formatted_code, formatter)?;
                        } else {
                            // Expr fits in a single line
                            write!(formatted_code, "{}", buf)?;
                        }

                        Ok(())
                    },
                )?;
            }
            Self::Asm(asm_block) => asm_block.format(formatted_code, formatter)?,
            Self::Return {
                return_token: _,
                expr_opt,
            } => {
                write!(formatted_code, "{}", ReturnToken::AS_STR)?;
                if let Some(expr) = &expr_opt {
                    write!(formatted_code, " ")?;
                    expr.format(formatted_code, formatter)?;
                }
            }
            Self::Panic {
                panic_token: _,
                expr_opt,
            } => {
                write!(formatted_code, "{}", PanicToken::AS_STR)?;
                if let Some(expr) = &expr_opt {
                    write!(formatted_code, " ")?;
                    expr.format(formatted_code, formatter)?;
                }
            }
            Self::If(if_expr) => if_expr.format(formatted_code, formatter)?,
            Self::Match {
                match_token: _,
                value,
                branches,
            } => {
                write!(formatted_code, "{} ", MatchToken::AS_STR)?;
                value.format(formatted_code, formatter)?;
                write!(formatted_code, " ")?;
                if !branches.get().is_empty() {
                    MatchBranch::open_curly_brace(formatted_code, formatter)?;
                    let branches = branches.get();
                    for match_branch in branches.iter() {
                        write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                        match_branch.format(formatted_code, formatter)?;
                        writeln!(formatted_code)?;
                    }
                    MatchBranch::close_curly_brace(formatted_code, formatter)?;
                } else {
                    write!(formatted_code, "{{}}")?;
                }
            }
            Self::While {
                while_token: _,
                condition,
                block,
            } => {
                formatter.with_shape(
                    formatter
                        .shape
                        .with_code_line_from(LineStyle::Normal, ExprKind::Function),
                    |formatter| -> Result<(), FormatterError> {
                        write!(formatted_code, "{} ", WhileToken::AS_STR)?;
                        condition.format(formatted_code, formatter)?;
                        IfExpr::open_curly_brace(formatted_code, formatter)?;
                        block.get().format(formatted_code, formatter)?;
                        IfExpr::close_curly_brace(formatted_code, formatter)?;
                        Ok(())
                    },
                )?;
            }
            Self::For {
                for_token: _,
                in_token: _,
                value_pattern,
                iterator,
                block,
            } => {
                formatter.with_shape(
                    formatter
                        .shape
                        .with_code_line_from(LineStyle::Normal, ExprKind::Function),
                    |formatter| -> Result<(), FormatterError> {
                        write!(formatted_code, "{} ", ForToken::AS_STR)?;
                        value_pattern.format(formatted_code, formatter)?;
                        write!(formatted_code, " {} ", InToken::AS_STR)?;
                        iterator.format(formatted_code, formatter)?;
                        IfExpr::open_curly_brace(formatted_code, formatter)?;
                        block.get().format(formatted_code, formatter)?;
                        IfExpr::close_curly_brace(formatted_code, formatter)?;
                        Ok(())
                    },
                )?;
            }
            Self::FuncApp { func, args } => {
                formatter.with_shape(
                    formatter
                        .shape
                        .with_code_line_from(LineStyle::Normal, ExprKind::Function),
                    |formatter| -> Result<(), FormatterError> {
                        // don't indent unless on new line
                        if formatted_code.ends_with('\n') {
                            write!(formatted_code, "{}", formatter.indent_to_str()?)?;
                        }
                        func.format(formatted_code, formatter)?;

                        Self::open_parenthesis(formatted_code, formatter)?;
                        let (_, args_str) = write_function_call_arguments(args.get(), formatter)?;
                        write!(formatted_code, "{}", args_str)?;
                        Self::close_parenthesis(formatted_code, formatter)?;

                        Ok(())
                    },
                )?;
            }
            Self::Index { target, arg } => {
                target.format(formatted_code, formatter)?;
                Self::open_square_bracket(formatted_code, formatter)?;
                arg.get().format(formatted_code, formatter)?;
                Self::close_square_bracket(formatted_code, formatter)?;
            }
            Self::MethodCall {
                target,
                dot_token,
                path_seg,
                contract_args_opt,
                args,
            } => {
                formatter.with_shape(
                    formatter.shape.with_default_code_line(),
                    |formatter| -> Result<(), FormatterError> {
                        // get the length in chars of the code_line in a single line format
                        let mut buf = FormattedCode::new();
                        let mut temp_formatter = Formatter::default();
                        temp_formatter
                            .shape
                            .code_line
                            .update_line_style(LineStyle::Inline);

                        let (function_call_length, args_inline) = format_method_call(
                            target,
                            dot_token,
                            path_seg,
                            contract_args_opt,
                            args,
                            &mut buf,
                            &mut temp_formatter,
                        )?;

                        // get the largest field size
                        let (field_width, body_width) = if args_inline {
                            (function_call_length, function_call_length)
                        } else if let Some(contract_args) = &contract_args_opt {
                            get_field_width(contract_args.get(), &mut formatter.clone())?
                        } else {
                            (0, 0)
                        };

                        // changes to the actual formatter
                        let expr_width = buf.chars().count();
                        formatter.shape.code_line.add_width(expr_width);
                        formatter.shape.code_line.update_expr_kind(ExprKind::Struct);
                        formatter.shape.get_line_style(
                            Some(field_width),
                            Some(body_width),
                            &formatter.config,
                        );

                        let _ = format_method_call(
                            target,
                            dot_token,
                            path_seg,
                            contract_args_opt,
                            args,
                            formatted_code,
                            formatter,
                        )?;

                        Ok(())
                    },
                )?;
            }
            Self::FieldProjection {
                target,
                dot_token: _,
                name,
            } => {
                let prev_length = formatted_code.len();
                target.format(formatted_code, formatter)?;
                let diff = formatted_code.len() - prev_length;
                if diff > 5 && formatter.shape.code_line.expr_new_line {
                    // The next next expression should be added onto a new line.
                    // The only exception is the previous element has fewer than
                    // 5 characters, in which case we can add the dot onto the
                    // same line (for example self.x will be rendered in the
                    // same line)
                    formatter.indent();
                    write!(
                        formatted_code,
                        "\n{}{}",
                        formatter.indent_to_str()?,
                        DotToken::AS_STR,
                    )?;
                    name.format(formatted_code, formatter)?;
                    formatter.unindent();
                } else {
                    write!(formatted_code, "{}", DotToken::AS_STR)?;
                    name.format(formatted_code, formatter)?;
                }
            }
            Self::TupleFieldProjection {
                target,
                dot_token: _,
                field,
                field_span: _,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}{}", DotToken::AS_STR, field)?;
            }
            Self::Ref {
                ampersand_token: _,
                mut_token,
                expr,
            } => {
                // TODO: Currently, the parser does not support taking
                //       references on references without spaces between
                //       ampersands. E.g., `&&&x` is not supported and must
                //       be written as `& & &x`.
                //       See: https://github.com/FuelLabs/sway/issues/6808
                //       Until this issue is fixed, we need this workaround
                //       in case of referenced expression `expr` being itself a
                //       reference.
                if !matches!(expr.as_ref(), Self::Ref { .. }) {
                    // TODO: Keep this code once the issue is fixed.
                    write!(formatted_code, "{}", AmpersandToken::AS_STR)?;
                    if mut_token.is_some() {
                        write!(formatted_code, "{} ", MutToken::AS_STR)?;
                    }
                    expr.format(formatted_code, formatter)?;
                } else {
                    // TODO: This is the workaround if `expr` is a reference.
                    write!(formatted_code, "{}", AmpersandToken::AS_STR)?;
                    // If we have the `mut`, we will also
                    // get a space after it, so the next `&`
                    // will be separated. Otherwise, insert space.
                    if mut_token.is_some() {
                        write!(formatted_code, "{} ", MutToken::AS_STR)?;
                    } else {
                        write!(formatted_code, " ")?;
                    }
                    expr.format(formatted_code, formatter)?;
                }
            }
            Self::Deref {
                star_token: _,
                expr,
            } => {
                write!(formatted_code, "{}", StarToken::AS_STR)?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Not {
                bang_token: _,
                expr,
            } => {
                write!(formatted_code, "{}", BangToken::AS_STR)?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Pow {
                lhs,
                double_star_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", DoubleStarToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Mul {
                lhs,
                star_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", StarToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Div {
                lhs,
                forward_slash_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", ForwardSlashToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Modulo {
                lhs,
                percent_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", PercentToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Add {
                lhs,
                add_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", AddToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Sub {
                lhs,
                sub_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", SubToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Shl {
                lhs,
                shl_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", ShlToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Shr {
                lhs,
                shr_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", ShrToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitAnd {
                lhs,
                ampersand_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                match formatter.shape.code_line.line_style {
                    LineStyle::Multiline => {
                        write!(
                            formatted_code,
                            "\n{}{} ",
                            formatter.indent_to_str()?,
                            AmpersandToken::AS_STR,
                        )?;
                    }
                    _ => {
                        write!(formatted_code, " {} ", AmpersandToken::AS_STR)?;
                    }
                }
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitXor {
                lhs,
                caret_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                match formatter.shape.code_line.line_style {
                    LineStyle::Multiline => {
                        write!(
                            formatted_code,
                            "\n{}{} ",
                            formatter.indent_to_str()?,
                            CaretToken::AS_STR,
                        )?;
                    }
                    _ => {
                        write!(formatted_code, " {} ", CaretToken::AS_STR)?;
                    }
                }
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitOr {
                lhs,
                pipe_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                match formatter.shape.code_line.line_style {
                    LineStyle::Multiline => {
                        write!(
                            formatted_code,
                            "\n{}{} ",
                            formatter.indent_to_str()?,
                            PipeToken::AS_STR,
                        )?;
                    }
                    _ => {
                        write!(formatted_code, " {} ", PipeToken::AS_STR)?;
                    }
                }
                rhs.format(formatted_code, formatter)?;
            }
            Self::Equal {
                lhs,
                double_eq_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", DoubleEqToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::NotEqual {
                lhs,
                bang_eq_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", BangEqToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LessThan {
                lhs,
                less_than_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", LessThanToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::GreaterThan {
                lhs,
                greater_than_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", GreaterThanToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LessThanEq {
                lhs,
                less_than_eq_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", LessThanEqToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::GreaterThanEq {
                lhs,
                greater_than_eq_token: _,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", GreaterThanEqToken::AS_STR)?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LogicalAnd {
                lhs,
                double_ampersand_token: _,
                rhs,
            } => {
                two_parts_expr(
                    lhs,
                    DoubleAmpersandToken::AS_STR,
                    rhs,
                    formatted_code,
                    formatter,
                )?;
            }
            Self::LogicalOr {
                lhs,
                double_pipe_token: _,
                rhs,
            } => {
                two_parts_expr(lhs, DoublePipeToken::AS_STR, rhs, formatted_code, formatter)?;
            }
            Self::Reassignment {
                assignable,
                reassignment_op,
                expr,
            } => {
                assignable.format(formatted_code, formatter)?;
                reassignment_op.format(formatted_code, formatter)?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Break { break_token: _ } => {
                write!(formatted_code, "{}", BreakToken::AS_STR)?;
            }
            Self::Continue { continue_token: _ } => {
                write!(formatted_code, "{}", ContinueToken::AS_STR)?;
            }
        }

        Ok(())
    }
}

impl Parenthesis for Expr {
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

impl SquareBracket for Expr {
    fn open_square_bracket(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_open_char())?;
        Ok(())
    }
    fn close_square_bracket(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_close_char())?;
        Ok(())
    }
}

pub(super) fn debug_expr(
    buf: FormattedCode,
    field_width: Option<usize>,
    body_width: Option<usize>,
    expr_width: usize,
    formatter: &Formatter,
) {
    println!(
        "DEBUG:\nline: {buf}\nfield: {:?}, body: {:?}, expr: {expr_width}, Shape::width: {}",
        field_width, body_width, formatter.shape.code_line.width
    );
    println!("{:?}", formatter.shape.code_line);
    println!("{:?}\n", formatter.shape.width_heuristics);
}

fn format_expr_struct(
    path: &PathExpr,
    fields: &Braces<Punctuated<ExprStructField, CommaToken>>,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    path.format(formatted_code, formatter)?;
    ExprStructField::open_curly_brace(formatted_code, formatter)?;
    let fields = &fields.get();
    match formatter.shape.code_line.line_style {
        LineStyle::Inline => fields.format(formatted_code, formatter)?,
        // TODO: add field alignment
        _ => fields.format(formatted_code, formatter)?,
    }
    ExprStructField::close_curly_brace(formatted_code, formatter)?;

    Ok(())
}

/// Checks if the current generated code is too long to fit into a single line
/// or it should be broken into multiple lines. The logic to break the
/// expression into multiple line is handled inside each struct.
///
/// Alternatively, if `expr_new_line` is set to true this function always will
/// return true
#[inline]
pub fn should_write_multiline(code: &str, formatter: &Formatter) -> bool {
    if formatter.shape.code_line.expr_new_line {
        true
    } else {
        let max_per_line = formatter.shape.width_heuristics.collection_width;
        for (i, c) in code.chars().rev().enumerate() {
            if c == '\n' {
                return i > max_per_line;
            }
        }

        false
    }
}

/// Whether this expression can be inlined if it is the sole argument of a
/// function/method call
#[inline]
fn same_line_if_only_argument(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Struct { path: _, fields: _ }
            | Expr::Tuple(_)
            | Expr::Array(_)
            | Expr::Parens(_)
            | Expr::Not {
                bang_token: _,
                expr: _
            }
            | Expr::Path(_)
            | Expr::FuncApp { func: _, args: _ }
            | Expr::Match {
                match_token: _,
                value: _,
                branches: _
            }
    )
}

#[inline]
pub(crate) fn is_single_argument_and_can_be_inline<P>(
    args: &Punctuated<Expr, P>,
    formatter: &mut Formatter,
) -> bool
where
    P: Format + std::fmt::Debug,
{
    formatter.with_shape(
        formatter
            .shape
            .with_code_line_from(LineStyle::Normal, ExprKind::Function),
        |formatter| -> bool {
            let mut buf = FormattedCode::new();
            if args.value_separator_pairs.len() == 1 && args.final_value_opt.is_none() {
                if same_line_if_only_argument(&args.value_separator_pairs[0].0) {
                    return true;
                }
                let _ = args.value_separator_pairs[0].0.format(&mut buf, formatter);
            } else if args.value_separator_pairs.is_empty() && args.final_value_opt.is_some() {
                if let Some(final_value) = &args.final_value_opt {
                    if same_line_if_only_argument(final_value) {
                        return true;
                    }
                    let _ = (**final_value).format(&mut buf, formatter);
                }
            } else {
                return false;
            }
            buf.len() < formatter.shape.width_heuristics.collection_width
        },
    )
}

/// Writes the `(args)` of a function call. This is a common abstraction for
/// methods and functions and how to organize their arguments.
#[inline]
pub fn write_function_call_arguments<P>(
    args: &Punctuated<Expr, P>,
    formatter: &mut Formatter,
) -> Result<(bool, String), FormatterError>
where
    P: Format + std::fmt::Debug,
{
    let has_single_argument_and_can_be_inlined =
        is_single_argument_and_can_be_inline(args, formatter);

    formatter.with_shape(
        formatter
            .shape
            .with_code_line_from(LineStyle::Normal, ExprKind::Function),
        |formatter| -> Result<(bool, String), FormatterError> {
            let mut buf = FormattedCode::new();
            args.format(&mut buf, formatter)?;

            Ok(if has_single_argument_and_can_be_inlined {
                (true, buf.trim().to_owned())
            } else {
                // Check if the arguments can fit on a single line
                let expr_width = buf.chars().count();
                formatter.shape.code_line.add_width(expr_width);
                formatter.shape.get_line_style(
                    Some(expr_width),
                    Some(expr_width),
                    &formatter.config,
                );

                if expr_width == 0 {
                    return Ok((true, "".to_owned()));
                }
                match formatter.shape.code_line.line_style {
                    LineStyle::Multiline => {
                        // force each param to be a new line
                        formatter.shape.code_line.update_expr_new_line(true);
                        formatter.indent();
                        // should be rewritten to a multi-line
                        let mut formatted_code = FormattedCode::new();
                        let mut buf = FormattedCode::new();
                        args.format(&mut buf, formatter)?;
                        formatter.unindent();
                        writeln!(formatted_code, "{}", buf.trim_end())?;
                        formatter.write_indent_into_buffer(&mut formatted_code)?;
                        (false, formatted_code)
                    }
                    _ => (true, buf.trim().to_owned()),
                }
            })
        },
    )
}

fn format_method_call(
    target: &Expr,
    _dot_token: &DotToken,
    path_seg: &PathExprSegment,
    contract_args_opt: &Option<Braces<Punctuated<ExprStructField, CommaToken>>>,
    args: &Parens<Punctuated<Expr, CommaToken>>,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(usize, bool), FormatterError> {
    // don't indent unless on new line
    if formatted_code.ends_with('\n') {
        write!(formatted_code, "{}", formatter.indent_to_str()?)?;
    }
    target.format(formatted_code, formatter)?;

    if formatter.shape.code_line.expr_new_line {
        formatter.indent();
        write!(formatted_code, "\n{}", formatter.indent_to_str()?)?;
    }

    write!(formatted_code, "{}", DotToken::AS_STR)?;

    path_seg.format(formatted_code, formatter)?;
    if let Some(contract_args) = &contract_args_opt {
        ExprStructField::open_curly_brace(formatted_code, formatter)?;
        let contract_args = &contract_args.get();
        match formatter.shape.code_line.line_style {
            LineStyle::Inline => {
                contract_args.format(formatted_code, formatter)?;
            }
            _ => {
                contract_args.format(formatted_code, formatter)?;
            }
        }
        ExprStructField::close_curly_brace(formatted_code, formatter)?;
    }

    let len_function_call = formatted_code.len();

    Expr::open_parenthesis(formatted_code, formatter)?;
    let (args_inline, args_str) = write_function_call_arguments(args.get(), formatter)?;
    write!(formatted_code, "{}", args_str)?;
    Expr::close_parenthesis(formatted_code, formatter)?;

    if formatter.shape.code_line.expr_new_line {
        formatter.unindent();
    }
    Ok((len_function_call, args_inline))
}

fn get_field_width(
    fields: &Punctuated<ExprStructField, CommaToken>,
    formatter: &mut Formatter,
) -> Result<(usize, usize), FormatterError> {
    let mut largest_field: usize = 0;
    let mut body_width: usize = 3; // this is taking into account the opening brace, the following space and the ending brace.
    for (field, _comma_token) in &fields.value_separator_pairs {
        let mut field_length = field.field_name.as_str().chars().count();
        if let Some((_colon_token, expr)) = &field.expr_opt {
            let mut buf = String::new();
            write!(buf, "{} ", ColonToken::AS_STR)?;
            expr.format(&mut buf, formatter)?;
            field_length += buf.chars().count();
        }
        field_length += CommaToken::AS_STR.chars().count();
        body_width += &field_length + 1; // accounting for the following space

        if field_length > largest_field {
            largest_field = field_length;
        }
    }
    if let Some(final_value) = &fields.final_value_opt {
        let mut field_length = final_value.field_name.as_str().chars().count();
        if let Some((_colon_token, expr)) = &final_value.expr_opt {
            let mut buf = String::new();
            write!(buf, "{} ", ColonToken::AS_STR)?;
            expr.format(&mut buf, formatter)?;
            field_length += buf.chars().count();
        }
        body_width += &field_length + 1; // accounting for the following space

        if field_length > largest_field {
            largest_field = field_length;
        }
    }

    Ok((largest_field, body_width))
}

// Leaf Spans

impl LeafSpans for Expr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        expr_leaf_spans(self)
    }
}

/// Collects various [Expr] field's [ByteSpan]s.
fn expr_leaf_spans(expr: &Expr) -> Vec<ByteSpan> {
    match expr {
        Expr::Error(_, _) => vec![expr.span().into()],
        Expr::Path(path) => path.leaf_spans(),
        Expr::Literal(literal) => literal.leaf_spans(),
        Expr::AbiCast { abi_token, args } => {
            let mut collected_spans = vec![ByteSpan::from(abi_token.span())];
            collected_spans.append(&mut args.leaf_spans());
            collected_spans
        }
        Expr::Struct { path, fields } => {
            let mut collected_spans = path.leaf_spans();
            collected_spans.append(&mut fields.leaf_spans());
            collected_spans
        }
        Expr::Tuple(tuple) => tuple.leaf_spans(),
        Expr::Parens(parens) => parens.leaf_spans(),
        Expr::Block(block) => block.leaf_spans(),
        Expr::Array(array) => array.leaf_spans(),
        Expr::Asm(asm) => asm.leaf_spans(),
        Expr::Return {
            return_token,
            expr_opt,
        } => {
            let mut collected_spans = vec![ByteSpan::from(return_token.span())];
            if let Some(expr) = expr_opt {
                collected_spans.append(&mut expr.leaf_spans());
            }
            collected_spans
        }
        Expr::Panic {
            panic_token,
            expr_opt,
        } => {
            let mut collected_spans = vec![ByteSpan::from(panic_token.span())];
            if let Some(expr) = expr_opt {
                collected_spans.append(&mut expr.leaf_spans());
            }
            collected_spans
        }
        Expr::If(expr_if) => expr_if.leaf_spans(),
        Expr::Match {
            match_token,
            value,
            branches,
        } => {
            let mut collected_spans = vec![ByteSpan::from(match_token.span())];
            collected_spans.append(&mut value.leaf_spans());
            collected_spans.append(&mut branches.leaf_spans());
            collected_spans
        }
        Expr::While {
            while_token,
            condition,
            block,
        } => {
            let mut collected_spans = vec![ByteSpan::from(while_token.span())];
            collected_spans.append(&mut condition.leaf_spans());
            collected_spans.append(&mut block.leaf_spans());
            collected_spans
        }
        Expr::For {
            for_token,
            in_token,
            value_pattern,
            iterator,
            block,
        } => {
            let mut collected_spans = vec![ByteSpan::from(for_token.span())];
            collected_spans.append(&mut value_pattern.leaf_spans());
            collected_spans.append(&mut vec![ByteSpan::from(in_token.span())]);
            collected_spans.append(&mut iterator.leaf_spans());
            collected_spans.append(&mut block.leaf_spans());
            collected_spans
        }
        Expr::FuncApp { func, args } => {
            let mut collected_spans = Vec::new();
            collected_spans.append(&mut func.leaf_spans());
            collected_spans.append(&mut args.leaf_spans());
            collected_spans
        }
        Expr::Index { target, arg } => {
            let mut collected_spans = Vec::new();
            collected_spans.append(&mut target.leaf_spans());
            collected_spans.append(&mut arg.leaf_spans());
            collected_spans
        }
        Expr::MethodCall {
            target,
            dot_token,
            path_seg,
            contract_args_opt,
            args,
        } => {
            let mut collected_spans = Vec::new();
            collected_spans.append(&mut target.leaf_spans());
            collected_spans.push(ByteSpan::from(dot_token.span()));
            collected_spans.push(ByteSpan::from(path_seg.span()));
            if let Some(contract_args) = contract_args_opt {
                collected_spans.append(&mut contract_args.leaf_spans());
            }
            collected_spans.append(&mut args.leaf_spans());
            collected_spans
        }
        Expr::FieldProjection {
            target,
            dot_token,
            name,
        } => {
            let mut collected_spans = Vec::new();
            collected_spans.append(&mut target.leaf_spans());
            collected_spans.push(ByteSpan::from(dot_token.span()));
            collected_spans.push(ByteSpan::from(name.span()));
            collected_spans
        }
        Expr::TupleFieldProjection {
            target,
            dot_token,
            field: _field,
            field_span,
        } => {
            let mut collected_spans = Vec::new();
            collected_spans.append(&mut target.leaf_spans());
            collected_spans.push(ByteSpan::from(dot_token.span()));
            collected_spans.push(ByteSpan::from(field_span.clone()));
            collected_spans
        }
        Expr::Ref {
            ampersand_token,
            mut_token,
            expr,
        } => {
            let mut collected_spans = vec![ByteSpan::from(ampersand_token.span())];
            if let Some(mut_token) = mut_token {
                collected_spans.push(ByteSpan::from(mut_token.span()));
            }
            collected_spans.append(&mut expr.leaf_spans());
            collected_spans
        }
        Expr::Deref { star_token, expr } => {
            let mut collected_spans = vec![ByteSpan::from(star_token.span())];
            collected_spans.append(&mut expr.leaf_spans());
            collected_spans
        }
        Expr::Not { bang_token, expr } => {
            let mut collected_spans = vec![ByteSpan::from(bang_token.span())];
            collected_spans.append(&mut expr.leaf_spans());
            collected_spans
        }
        Expr::Pow {
            lhs,
            double_star_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(double_star_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Mul {
            lhs,
            star_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(star_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Div {
            lhs,
            forward_slash_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(forward_slash_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Modulo {
            lhs,
            percent_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(percent_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Add {
            lhs,
            add_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(add_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Sub {
            lhs,
            sub_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(sub_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Shl {
            lhs,
            shl_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(shl_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Shr {
            lhs,
            shr_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(shr_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::BitAnd {
            lhs,
            ampersand_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(ampersand_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::BitXor {
            lhs,
            caret_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(caret_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::BitOr {
            lhs,
            pipe_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(pipe_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(double_eq_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(bang_eq_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(less_than_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(greater_than_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(less_than_eq_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(greater_than_eq_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::LogicalAnd {
            lhs,
            double_ampersand_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(double_ampersand_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::LogicalOr {
            lhs,
            double_pipe_token,
            rhs,
        } => {
            let mut collected_spans = lhs.leaf_spans();
            collected_spans.push(ByteSpan::from(double_pipe_token.span()));
            collected_spans.append(&mut rhs.leaf_spans());
            collected_spans
        }
        Expr::Reassignment {
            assignable,
            reassignment_op,
            expr,
        } => {
            let mut collected_spans = assignable.leaf_spans();
            collected_spans.push(ByteSpan::from(reassignment_op.span.clone()));
            collected_spans.append(&mut expr.leaf_spans());
            collected_spans
        }
        Expr::Break { break_token } => {
            vec![ByteSpan::from(break_token.span())]
        }
        Expr::Continue { continue_token } => {
            vec![ByteSpan::from(continue_token.span())]
        }
    }
}
