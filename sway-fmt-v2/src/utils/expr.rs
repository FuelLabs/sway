use crate::{
    config::items::ItemBraceStyle,
    fmt::*,
    utils::comments::{ByteSpan, LeafSpans},
};
use std::{fmt::Write, vec};
use sway_parse::{
    expr::asm::{AsmBlockContents, AsmFinalExpr},
    token::{Delimiter, PunctKind},
    AbiCastArgs, AsmBlock, AsmRegisterDeclaration, Assignable, Expr, ExprArrayDescriptor,
    ExprStructField, ExprTupleDescriptor, IfCondition, IfExpr, Instruction, MatchBranch,
    MatchBranchKind,
};
use sway_types::Spanned;

use super::bracket::CurlyBrace;

// TODO:
impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            //     Self::Path(path) => {}
            //     Self::Literal(lit) => {}
            //     Self::AbiCast { abi_token, args } => {}
            Self::Struct { path, fields } => {
                path.format(formatted_code, formatter)?;
                ExprStructField::open_curly_brace(formatted_code, formatter)?;
                writeln!(formatted_code)?;
                let fields = fields.clone().into_inner();
                let mut value_pairs_iter = fields.value_separator_pairs.iter().peekable();
                for (expr_struct_field, comma_token) in value_pairs_iter.clone() {
                    // TypeField
                    expr_struct_field.format(formatted_code, formatter)?;

                    if value_pairs_iter.peek().is_some() {
                        writeln!(formatted_code, "{}", comma_token.ident().as_str())?;
                    }
                }
                if let Some(final_value) = &fields.final_value_opt {
                    write!(
                        formatted_code,
                        "{}",
                        &formatter.shape.indent.to_string(&formatter.config)?
                    )?;
                    final_value.format(formatted_code, formatter)?;
                    writeln!(formatted_code, "{}", PunctKind::Comma.as_char())?;
                }
                ExprStructField::close_curly_brace(formatted_code, formatter)?;
            }
            //     Self::Tuple(tuple_descriptor) => {}
            //     Self::Parens(expr) => {}
            //     Self::Block(code_block) => {}
            //     Self::Array(array_descriptor) => {}
            //     Self::Asm(asm_block) => {}
            //     Self::Return {
            //         return_token,
            //         expr_opt,
            //     } => {}
            //     Self::If(if_expr) => {}
            //     Self::Match {
            //         match_token,
            //         value,
            //         branches,
            //     } => {}
            //     Self::While {
            //         while_token,
            //         condition,
            //         block,
            //     } => {}
            //     Self::FuncApp { func, args } => {}
            //     Self::Index { target, arg } => {}
            //     Self::MethodCall {
            //         target,
            //         dot_token,
            //         name,
            //         contract_args_opt,
            //         args,
            //     } => {}
            //     Self::FieldProjection {
            //         target,
            //         dot_token,
            //         name,
            //     } => {}
            //     Self::TupleFieldProjection {
            //         target,
            //         dot_token,
            //         field,
            //         field_span,
            //     } => {}
            //     Self::Ref { ref_token, expr } => {}
            //     Self::Deref { deref_token, expr } => {}
            //     Self::Not { bang_token, expr } => {}
            //     Self::Mul {
            //         lhs,
            //         star_token,
            //         rhs,
            //     } => {}
            //     Self::Div {
            //         lhs,
            //         forward_slash_token,
            //         rhs,
            //     } => {}
            //     Self::Modulo {
            //         lhs,
            //         percent_token,
            //         rhs,
            //     } => {}
            //     Self::Add {
            //         lhs,
            //         add_token,
            //         rhs,
            //     } => {}
            //     Self::Sub {
            //         lhs,
            //         sub_token,
            //         rhs,
            //     } => {}
            //     Self::Shl {
            //         lhs,
            //         shl_token,
            //         rhs,
            //     } => {}
            //     Self::Shr {
            //         lhs,
            //         shr_token,
            //         rhs,
            //     } => {}
            //     Self::BitAnd {
            //         lhs,
            //         ampersand_token,
            //         rhs,
            //     } => {}
            //     Self::BitXor {
            //         lhs,
            //         caret_token,
            //         rhs,
            //     } => {}
            //     Self::BitOr {
            //         lhs,
            //         pipe_token,
            //         rhs,
            //     } => {}
            //     Self::Equal {
            //         lhs,
            //         double_eq_token,
            //         rhs,
            //     } => {}
            //     Self::NotEqual {
            //         lhs,
            //         bang_eq_token,
            //         rhs,
            //     } => {}
            //     Self::LessThan {
            //         lhs,
            //         less_than_token,
            //         rhs,
            //     } => {}
            //     Self::GreaterThan {
            //         lhs,
            //         greater_than_token,
            //         rhs,
            //     } => {}
            //     Self::LessThanEq {
            //         lhs,
            //         less_than_eq_token,
            //         rhs,
            //     } => {}
            //     Self::GreaterThanEq {
            //         lhs,
            //         greater_than_eq_token,
            //         rhs,
            //     } => {}
            //     Self::LogicalAnd {
            //         lhs,
            //         double_ampersand_token,
            //         rhs,
            //     } => {}
            //     Self::LogicalOr {
            //         lhs,
            //         double_pipe_token,
            //         rhs,
            //     } => {}
            //     Self::Reassignment {
            //         assignable,
            //         reassignment_op,
            //         expr,
            //     } => {}
            _ => write!(formatted_code, "{}", self.span().as_str())?,
        }

        Ok(())
    }
}

impl Format for ExprStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(
            formatted_code,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            self.field_name.span().as_str()
        )?;
        if let Some((colon_token, expr)) = &self.expr_opt {
            write!(formatted_code, "{} ", colon_token.ident().as_str())?;
            expr.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}

impl CurlyBrace for ExprStructField {
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
        write!(
            line,
            "{}{}",
            formatter.shape.indent.to_string(&formatter.config)?,
            Delimiter::Brace.as_close_char()
        )?;
        Ok(())
    }
}

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
    }
}

impl LeafSpans for AbiCastArgs {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.name.span())];
        collected_spans.push(ByteSpan::from(self.comma_token.span()));
        collected_spans.append(&mut self.address.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for ExprStructField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.field_name.span())];
        if let Some(expr) = &self.expr_opt {
            collected_spans.push(ByteSpan::from(expr.0.span()));
            collected_spans.append(&mut expr.1.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for ExprTupleDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let ExprTupleDescriptor::Cons {
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

impl LeafSpans for ExprArrayDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let ExprArrayDescriptor::Repeat {
            value,
            semicolon_token,
            length,
        } = self
        {
            collected_spans.append(&mut value.leaf_spans());
            collected_spans.push(ByteSpan::from(semicolon_token.span()));
            collected_spans.append(&mut length.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for AsmBlock {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.asm_token.span())];
        collected_spans.append(&mut self.registers.leaf_spans());
        collected_spans.append(&mut self.contents.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for AsmRegisterDeclaration {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.register.span())];
        if let Some(value) = &self.value_opt {
            collected_spans.append(&mut value.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for AsmBlockContents {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        for instruction in &self.instructions {
            collected_spans.append(&mut instruction.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for Instruction {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        // Visit instructions as a whole unit, meaning we cannot insert comments inside an instruction.
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for AsmFinalExpr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.register.span())];
        if let Some(ty) = &self.ty_opt {
            collected_spans.append(&mut ty.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for IfExpr {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.if_token.span())];
        collected_spans.append(&mut self.condition.leaf_spans());
        collected_spans.append(&mut self.then_block.leaf_spans());
        if let Some(else_block) = &self.else_opt {
            collected_spans.push(ByteSpan::from(else_block.0.span()));
            let mut else_body_spans = match &else_block.1 {
                std::ops::ControlFlow::Continue(if_expr) => if_expr.leaf_spans(),
                std::ops::ControlFlow::Break(else_body) => else_body.leaf_spans(),
            };
            collected_spans.append(&mut else_body_spans);
        }
        collected_spans
    }
}

impl LeafSpans for IfCondition {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            IfCondition::Expr(expr) => expr.leaf_spans(),
            IfCondition::Let {
                let_token,
                lhs,
                eq_token,
                rhs,
            } => {
                let mut collected_spans = vec![ByteSpan::from(let_token.span())];
                collected_spans.append(&mut lhs.leaf_spans());
                collected_spans.push(ByteSpan::from(eq_token.span()));
                collected_spans.append(&mut rhs.leaf_spans());
                collected_spans
            }
        }
    }
}

impl LeafSpans for MatchBranch {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.append(&mut self.pattern.leaf_spans());
        collected_spans.push(ByteSpan::from(self.fat_right_arrow_token.span()));
        collected_spans.append(&mut self.kind.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for MatchBranchKind {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            MatchBranchKind::Block {
                block,
                comma_token_opt,
            } => {
                collected_spans.append(&mut block.leaf_spans());
                if let Some(comma_token) = comma_token_opt {
                    collected_spans.push(ByteSpan::from(comma_token.span()));
                }
            }
            MatchBranchKind::Expr { expr, comma_token } => {
                collected_spans.append(&mut expr.leaf_spans());
                collected_spans.push(ByteSpan::from(comma_token.span()));
            }
        };
        collected_spans
    }
}

impl LeafSpans for Assignable {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            Assignable::Var(var) => collected_spans.push(ByteSpan::from(var.span())),
            Assignable::Index { target, arg } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.append(&mut arg.leaf_spans());
            }
            Assignable::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.push(ByteSpan::from(dot_token.span()));
                collected_spans.push(ByteSpan::from(name.span()));
            }
            Assignable::TupleFieldProjection {
                target,
                dot_token,
                field: _field,
                field_span,
            } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.push(ByteSpan::from(dot_token.span()));
                collected_spans.push(ByteSpan::from(field_span.clone()));
            }
        };
        collected_spans
    }
}
