use crate::{
    fmt::*,
    utils::{
        bracket::{CurlyBrace, Parenthesis, SquareBracket},
        comments::{ByteSpan, LeafSpans},
        shape::{ExprKind, LineStyle},
    },
};
use std::fmt::Write;
use sway_ast::{
    brackets::Parens,
    keywords::{CommaToken, DotToken},
    punctuated::Punctuated,
    token::Delimiter,
    Braces, CodeBlockContents, Expr, ExprArrayDescriptor, ExprStructField, ExprTupleDescriptor,
    MatchBranch, PathExpr,
};
use sway_types::{Ident, Spanned};

pub(crate) mod abi_cast;
pub(crate) mod asm_block;
pub(crate) mod assignable;
pub(crate) mod code_block;
pub(crate) mod collections;
pub(crate) mod conditional;
pub(crate) mod struct_field;

#[cfg(test)]
mod tests;

impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Path(path) => path.format(formatted_code, formatter)?,
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::AbiCast { abi_token, args } => {
                write!(formatted_code, "{}", abi_token.span().as_str())?;
                args.get().format(formatted_code, formatter)?;
            }
            Self::Struct { path, fields } => {
                // store previous state and update expr kind
                let prev_state = formatter.shape.code_line;
                formatter.shape.code_line.update_expr_kind(ExprKind::Struct);

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

                // changes to the actual formatter
                let expr_width = buf.chars().count() as usize;
                formatter.shape.add_width(expr_width);
                formatter.shape.get_line_style(
                    Some(field_width),
                    Some(body_width),
                    &formatter.config,
                );
                debug_expr(buf, field_width, body_width, expr_width, formatter);

                format_expr_struct(path, fields, formatted_code, formatter)?;

                // revert to previous state
                formatter.shape.sub_width(expr_width);
                formatter.shape.update_line_settings(prev_state);
            }
            Self::Tuple(tuple_descriptor) => {
                // store previous state and update expr kind
                let prev_state = formatter.shape.code_line;
                formatter
                    .shape
                    .code_line
                    .update_expr_kind(ExprKind::Collection);

                // get the length in chars of the code_line in a normal line format
                let mut buf = FormattedCode::new();
                let mut temp_formatter = Formatter::default();
                format_tuple(tuple_descriptor, &mut buf, &mut temp_formatter)?;
                let body_width = buf.chars().count() as usize;

                formatter.shape.add_width(body_width);
                formatter
                    .shape
                    .get_line_style(None, Some(body_width), &formatter.config);

                format_tuple(tuple_descriptor, formatted_code, formatter)?;

                // revert to previous state
                formatter.shape.sub_width(body_width);
                formatter.shape.update_line_settings(prev_state);
            }
            Self::Parens(expr) => {
                Self::open_parenthesis(formatted_code, formatter)?;
                expr.get().format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Block(code_block) => {
                CodeBlockContents::open_curly_brace(formatted_code, formatter)?;
                code_block.get().format(formatted_code, formatter)?;
                CodeBlockContents::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Array(array_descriptor) => {
                ExprArrayDescriptor::open_square_bracket(formatted_code, formatter)?;
                array_descriptor.get().format(formatted_code, formatter)?;
                ExprArrayDescriptor::close_square_bracket(formatted_code, formatter)?;
            }
            Self::Asm(asm_block) => asm_block.format(formatted_code, formatter)?,
            Self::Return {
                return_token,
                expr_opt,
            } => {
                write!(formatted_code, "{}", return_token.span().as_str())?;
                if let Some(expr) = &expr_opt {
                    write!(formatted_code, " ")?;
                    expr.format(formatted_code, formatter)?;
                }
            }
            Self::If(if_expr) => if_expr.format(formatted_code, formatter)?,
            Self::Match {
                match_token,
                value,
                branches,
            } => {
                write!(formatted_code, "{} ", match_token.span().as_str())?;
                value.format(formatted_code, formatter)?;
                MatchBranch::open_curly_brace(formatted_code, formatter)?;
                let branches = branches.get();
                for match_branch in branches.iter() {
                    match_branch.format(formatted_code, formatter)?;
                }
                MatchBranch::close_curly_brace(formatted_code, formatter)?;
            }
            Self::While {
                while_token,
                condition,
                block,
            } => {
                write!(formatted_code, "{} ", while_token.span().as_str())?;
                condition.format(formatted_code, formatter)?;
                CodeBlockContents::open_curly_brace(formatted_code, formatter)?;
                block.get().format(formatted_code, formatter)?;
                CodeBlockContents::close_curly_brace(formatted_code, formatter)?;
            }
            Self::FuncApp { func, args } => {
                // don't indent unless on new line
                if formatted_code.ends_with('\n') {
                    write!(
                        formatted_code,
                        "{}",
                        formatter.shape.indent.to_string(&formatter.config)?
                    )?;
                }
                func.format(formatted_code, formatter)?;
                Self::open_parenthesis(formatted_code, formatter)?;
                args.get().format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
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
                name,
                contract_args_opt,
                args,
            } => {
                let prev_state = formatter.shape.code_line;
                // get the length in chars of the code_line in a single line format
                let mut buf = FormattedCode::new();
                let mut temp_formatter = Formatter::default();
                temp_formatter
                    .shape
                    .code_line
                    .update_line_style(LineStyle::Inline);
                format_method_call(
                    target,
                    dot_token,
                    name,
                    contract_args_opt,
                    args,
                    &mut buf,
                    &mut temp_formatter,
                )?;

                // get the largest field size
                let (mut field_width, mut body_width): (usize, usize) = (0, 0);
                if let Some(contract_args) = &contract_args_opt {
                    (field_width, body_width) =
                        get_field_width(contract_args.get(), &mut formatter.clone())?;
                }

                // changes to the actual formatter
                let expr_width = buf.chars().count() as usize;
                formatter.shape.add_width(expr_width);
                formatter.shape.code_line.update_expr_kind(ExprKind::Struct);
                formatter.shape.get_line_style(
                    Some(field_width),
                    Some(body_width),
                    &formatter.config,
                );

                format_method_call(
                    target,
                    dot_token,
                    name,
                    contract_args_opt,
                    args,
                    formatted_code,
                    formatter,
                )?;

                // revert to previous state
                formatter.shape.sub_width(expr_width);
                formatter.shape.update_line_settings(prev_state);
            }
            Self::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", dot_token.span().as_str())?;
                name.format(formatted_code, formatter)?;
            }
            Self::TupleFieldProjection {
                target,
                dot_token,
                field: _,
                field_span,
            } => {
                target.format(formatted_code, formatter)?;
                write!(
                    formatted_code,
                    "{}{}",
                    dot_token.span().as_str(),
                    field_span.as_str(),
                )?;
            }
            Self::Ref { ref_token, expr } => {
                write!(formatted_code, "{} ", ref_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Deref { deref_token, expr } => {
                write!(formatted_code, "{} ", deref_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Not { bang_token, expr } => {
                write!(formatted_code, "{}", bang_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
            }
            Self::Mul {
                lhs,
                star_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", star_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Div {
                lhs,
                forward_slash_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", forward_slash_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Modulo {
                lhs,
                percent_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", percent_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Add {
                lhs,
                add_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", add_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Sub {
                lhs,
                sub_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", sub_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Shl {
                lhs,
                shl_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", shl_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Shr {
                lhs,
                shr_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", shr_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitAnd {
                lhs,
                ampersand_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", ampersand_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitXor {
                lhs,
                caret_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", caret_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::BitOr {
                lhs,
                pipe_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", pipe_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::Equal {
                lhs,
                double_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", double_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::NotEqual {
                lhs,
                bang_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", bang_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LessThan {
                lhs,
                less_than_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", less_than_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::GreaterThan {
                lhs,
                greater_than_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", greater_than_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LessThanEq {
                lhs,
                less_than_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", less_than_eq_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::GreaterThanEq {
                lhs,
                greater_than_eq_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(
                    formatted_code,
                    " {} ",
                    greater_than_eq_token.span().as_str()
                )?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LogicalAnd {
                lhs,
                double_ampersand_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(
                    formatted_code,
                    " {} ",
                    double_ampersand_token.span().as_str()
                )?;
                rhs.format(formatted_code, formatter)?;
            }
            Self::LogicalOr {
                lhs,
                double_pipe_token,
                rhs,
            } => {
                lhs.format(formatted_code, formatter)?;
                write!(formatted_code, " {} ", double_pipe_token.span().as_str())?;
                rhs.format(formatted_code, formatter)?;
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
            Self::Break { .. } => write!(formatted_code, "break")?,
            Self::Continue { .. } => write!(formatted_code, "continue")?,
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
    field_width: usize,
    body_width: usize,
    expr_width: usize,
    formatter: &mut Formatter,
) {
    println!(
        "line: {buf}\nfield: {field_width}, body: {body_width}, expr: {expr_width}, width: {}",
        formatter.shape.width
    );
    println!("{:?}", formatter.shape.code_line);
    println!("{:?}", formatter.shape.width_heuristics);
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

fn format_tuple(
    tuple_descriptor: &Parens<ExprTupleDescriptor>,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    tuple_descriptor.get().format(formatted_code, formatter)?;

    Ok(())
}

fn format_method_call(
    target: &Expr,
    dot_token: &DotToken,
    name: &Ident,
    contract_args_opt: &Option<Braces<Punctuated<ExprStructField, CommaToken>>>,
    args: &Parens<Punctuated<Expr, CommaToken>>,
    formatted_code: &mut FormattedCode,
    formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    // don't indent unless on new line
    if formatted_code.ends_with('\n') {
        write!(
            formatted_code,
            "{}",
            formatter.shape.indent.to_string(&formatter.config)?
        )?;
    }
    target.format(formatted_code, formatter)?;
    write!(formatted_code, "{}", dot_token.span().as_str())?;
    name.format(formatted_code, formatter)?;
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
    Expr::open_parenthesis(formatted_code, formatter)?;
    formatter.shape.reset_line_settings();
    args.get().format(formatted_code, formatter)?;
    Expr::close_parenthesis(formatted_code, formatter)?;

    Ok(())
}

fn get_field_width(
    fields: &Punctuated<ExprStructField, CommaToken>,
    formatter: &mut Formatter,
) -> Result<(usize, usize), FormatterError> {
    let mut largest_field: usize = 0;
    let mut body_width: usize = 3; // this is taking into account the opening brace, the following space and the ending brace.
    for (field, comma_token) in &fields.value_separator_pairs {
        let mut field_length = field.field_name.as_str().chars().count() as usize;
        if let Some((colon_token, expr)) = &field.expr_opt {
            let mut buf = String::new();
            write!(buf, "{} ", colon_token.span().as_str())?;
            expr.format(&mut buf, formatter)?;
            field_length += buf.chars().count() as usize;
        }
        field_length += comma_token.span().as_str().chars().count() as usize;
        body_width += &field_length + 1; // accounting for the following space

        if field_length > largest_field {
            largest_field = field_length;
        }
    }
    if let Some(final_value) = &fields.final_value_opt {
        let mut field_length = final_value.field_name.as_str().chars().count() as usize;
        if let Some((colon_token, expr)) = &final_value.expr_opt {
            let mut buf = String::new();
            write!(buf, "{} ", colon_token.span().as_str())?;
            expr.format(&mut buf, formatter)?;
            field_length += buf.chars().count() as usize;
        }
        body_width += &field_length + 1; // accounting for the following space

        if field_length > largest_field {
            largest_field = field_length;
        }
    }

    Ok((largest_field, body_width))
}

// Leaf Spans

// TODO: Find a better way of handling Boxed version
impl LeafSpans for Box<Expr> {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        visit_expr(self)
    }
}

impl LeafSpans for Expr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        visit_expr(self)
    }
}

/// Collects various expr field's ByteSpans.
fn visit_expr(expr: &Expr) -> Vec<ByteSpan> {
    match expr {
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
            name,
            contract_args_opt,
            args,
        } => {
            let mut collected_spans = Vec::new();
            collected_spans.append(&mut target.leaf_spans());
            collected_spans.push(ByteSpan::from(dot_token.span()));
            collected_spans.push(ByteSpan::from(name.span()));
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
        Expr::Ref { ref_token, expr } => {
            let mut collected_spans = vec![ByteSpan::from(ref_token.span())];
            collected_spans.append(&mut expr.leaf_spans());
            collected_spans
        }
        Expr::Deref { deref_token, expr } => {
            let mut collected_spans = vec![ByteSpan::from(deref_token.span())];
            collected_spans.append(&mut expr.leaf_spans());
            collected_spans
        }
        Expr::Not { bang_token, expr } => {
            let mut collected_spans = vec![ByteSpan::from(bang_token.span())];
            collected_spans.append(&mut expr.leaf_spans());
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
