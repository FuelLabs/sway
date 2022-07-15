use crate::{
    fmt::*,
    utils::comments::{CommentSpan, CommentVisitor},
};
use std::{fmt::Write, vec};
use sway_parse::{
    expr::asm::{AsmBlockContents, AsmFinalExpr},
    AbiCastArgs, AsmBlock, AsmRegisterDeclaration, Assignable, Expr, ExprArrayDescriptor,
    ExprStructField, ExprTupleDescriptor, IfCondition, IfExpr, Instruction, MatchBranch,
    MatchBranchKind,
};
use sway_types::Spanned;

#[allow(unused_variables)]
// TODO:
impl Format for Expr {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.span().as_str())?;
        // match self {
        //     Self::Path(path) => {}
        //     Self::Literal(lit) => {}
        //     Self::AbiCast { abi_token, args } => {}
        //     Self::Struct { path, fields } => {}
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
        // }

        Ok(())
    }
}
// TODO: Find a better way of handling Boxed version
impl CommentVisitor for Box<Expr> {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        visit_expr(self)
    }
}

impl CommentVisitor for Expr {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        visit_expr(self)
    }
}

pub fn visit_expr(expr: &Expr) -> Vec<CommentSpan> {
    match expr {
        Expr::Path(path) => path.collect_spans(),
        Expr::Literal(literal) => literal.collect_spans(),
        Expr::AbiCast { abi_token, args } => {
            let mut collected_spans = vec![CommentSpan::from_span(abi_token.span())];
            collected_spans.append(&mut args.collect_spans());
            collected_spans
        }
        Expr::Struct { path, fields } => {
            let mut collected_spans = path.collect_spans();
            collected_spans.append(&mut fields.collect_spans());
            collected_spans
        }
        Expr::Tuple(tuple) => tuple.collect_spans(),
        Expr::Parens(parens) => parens.collect_spans(),
        Expr::Block(block) => block.collect_spans(),
        Expr::Array(array) => array.collect_spans(),
        Expr::Asm(asm) => asm.collect_spans(),
        Expr::Return {
            return_token,
            expr_opt,
        } => {
            let mut collected_spans = vec![CommentSpan::from_span(return_token.span())];
            if let Some(expr) = expr_opt {
                collected_spans.append(&mut expr.collect_spans());
            }
            collected_spans
        }
        Expr::If(expr_if) => expr_if.collect_spans(),
        Expr::Match {
            match_token,
            value,
            branches,
        } => {
            let mut collected_spans = vec![CommentSpan::from_span(match_token.span())];
            // Collect value's CommentSpans
            collected_spans.append(&mut value.collect_spans());
            // Collect branches' CommentSpans
            collected_spans.append(&mut branches.collect_spans());
            collected_spans
        }
        Expr::While {
            while_token,
            condition,
            block,
        } => {
            let mut collected_spans = vec![CommentSpan::from_span(while_token.span())];
            // Collect condition's CommentSpans
            collected_spans.append(&mut condition.collect_spans());
            // Colelct block's CommentSpans
            collected_spans.append(&mut block.collect_spans());
            collected_spans
        }
        Expr::FuncApp { func, args } => {
            let mut collected_spans = Vec::new();
            // Collect func's CommentSpans
            collected_spans.append(&mut func.collect_spans());
            // Collect args' CommentSpans
            collected_spans.append(&mut args.collect_spans());
            collected_spans
        }
        Expr::Index { target, arg } => {
            let mut collected_spans = Vec::new();
            // Collect target's CommentSpans
            collected_spans.append(&mut target.collect_spans());
            // Collect arg's CommentSpans
            collected_spans.append(&mut arg.collect_spans());
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
            // Collect target's CommentSpans
            collected_spans.append(&mut target.collect_spans());
            // Add dot_token's CommentSpan
            collected_spans.push(CommentSpan::from_span(dot_token.span()));
            // Add name's CommentSpan
            collected_spans.push(CommentSpan::from_span(name.span()));
            // Collect contract args if it exists
            if let Some(contract_args) = contract_args_opt {
                collected_spans.append(&mut contract_args.collect_spans());
            }
            // Collect args CommentSpans
            collected_spans.append(&mut args.collect_spans());
            collected_spans
        }
        Expr::FieldProjection {
            target,
            dot_token,
            name,
        } => {
            let mut collected_spans = Vec::new();
            // Collect target's CommentSpans
            collected_spans.append(&mut target.collect_spans());
            // Add dot_token's CommentSpan
            collected_spans.push(CommentSpan::from_span(dot_token.span()));
            // Add name's CommentSpan
            collected_spans.push(CommentSpan::from_span(name.span()));
            collected_spans
        }
        Expr::TupleFieldProjection {
            target,
            dot_token,
            field: _field,
            field_span,
        } => {
            let mut collected_spans = Vec::new();
            // Collect target CommentSpans
            collected_spans.append(&mut target.collect_spans());
            // Add dot_token's CommentSpan
            collected_spans.push(CommentSpan::from_span(dot_token.span()));
            // Add field's CommentSpan
            collected_spans.push(CommentSpan::from_span(field_span.clone()));
            collected_spans
        }
        Expr::Ref { ref_token, expr } => {
            let mut collected_spans = vec![CommentSpan::from_span(ref_token.span())];
            collected_spans.append(&mut expr.collect_spans());
            collected_spans
        }
        Expr::Deref { deref_token, expr } => {
            let mut collected_spans = vec![CommentSpan::from_span(deref_token.span())];
            collected_spans.append(&mut expr.collect_spans());
            collected_spans
        }
        Expr::Not { bang_token, expr } => {
            let mut collected_spans = vec![CommentSpan::from_span(bang_token.span())];
            collected_spans.append(&mut expr.collect_spans());
            collected_spans
        }
        Expr::Mul {
            lhs,
            star_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(star_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Div {
            lhs,
            forward_slash_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(forward_slash_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Modulo {
            lhs,
            percent_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(percent_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Add {
            lhs,
            add_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(add_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Sub {
            lhs,
            sub_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(sub_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Shl {
            lhs,
            shl_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(shl_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Shr {
            lhs,
            shr_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(shr_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::BitAnd {
            lhs,
            ampersand_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(ampersand_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::BitXor {
            lhs,
            caret_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(caret_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::BitOr {
            lhs,
            pipe_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(pipe_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(double_eq_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(bang_eq_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(less_than_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(greater_than_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(less_than_eq_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(greater_than_eq_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::LogicalAnd {
            lhs,
            double_ampersand_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(double_ampersand_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::LogicalOr {
            lhs,
            double_pipe_token,
            rhs,
        } => {
            let mut collected_spans = lhs.collect_spans();
            collected_spans.push(CommentSpan::from_span(double_pipe_token.span()));
            collected_spans.append(&mut rhs.collect_spans());
            collected_spans
        }
        Expr::Reassignment {
            assignable,
            reassignment_op,
            expr,
        } => {
            let mut collected_spans = assignable.collect_spans();
            collected_spans.push(CommentSpan::from_span(reassignment_op.span.clone()));
            collected_spans.append(&mut expr.collect_spans());
            collected_spans
        }
    }
}

impl CommentVisitor for AbiCastArgs {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Add name's CommentSpan
        let mut collected_spans = vec![CommentSpan::from_span(self.name.span())];
        // Add comma_token's CommentSpan
        collected_spans.push(CommentSpan::from_span(self.comma_token.span()));
        // Add address CommentSpan
        collected_spans.append(&mut self.address.collect_spans());
        collected_spans
    }
}

impl CommentVisitor for ExprStructField {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Add field name's CommentSpan
        let mut collected_spans = vec![CommentSpan::from_span(self.field_name.span())];
        // Add expr's CommentSpan if it exists
        if let Some(expr) = &self.expr_opt {
            collected_spans.push(CommentSpan::from_span(expr.0.span()));
            // TODO: determine if we are allowing comments between `:` and expr
            collected_spans.append(&mut expr.1.collect_spans());
        }
        collected_spans
    }
}

impl CommentVisitor for ExprTupleDescriptor {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        if let ExprTupleDescriptor::Cons {
            head,
            comma_token,
            tail,
        } = self
        {
            // Collect head's CommentSpans
            collected_spans.append(&mut head.collect_spans());
            // Add comma_token's CommentSpan
            collected_spans.push(CommentSpan::from_span(comma_token.span()));
            // Collect tail's CommentSpans
            collected_spans.append(&mut tail.collect_spans());
        }
        collected_spans
    }
}

impl CommentVisitor for ExprArrayDescriptor {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        if let ExprArrayDescriptor::Repeat {
            value,
            semicolon_token,
            length,
        } = self
        {
            // Collect value's CommentSpans
            collected_spans.append(&mut value.collect_spans());
            // Add semicolon_token's CommentSpan
            collected_spans.push(CommentSpan::from_span(semicolon_token.span()));
            // Collect length's CommentSpans
            collected_spans.append(&mut length.collect_spans());
        }
        collected_spans
    }
}

impl CommentVisitor for AsmBlock {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Add asm_token's CommentSpan
        let mut collected_spans = vec![CommentSpan::from_span(self.asm_token.span())];
        // Collect registers' CommentSpans
        collected_spans.append(&mut self.registers.collect_spans());
        // Collect contents' CommentSpans
        collected_spans.append(&mut self.contents.collect_spans());
        collected_spans
    }
}

impl CommentVisitor for AsmRegisterDeclaration {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Add register's CommentSpan
        let mut collected_spans = vec![CommentSpan::from_span(self.register.span())];
        // Add value's CommentSpan if it exists
        if let Some(value) = &self.value_opt {
            collected_spans.push(CommentSpan::from_span(value.0.span()));
            // TODO: determine if we are allowing comments between `:` and expr
            collected_spans.append(&mut value.1.collect_spans());
        }
        collected_spans
    }
}

impl CommentVisitor for AsmBlockContents {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        for instruction in &self.instructions {
            // Add instruction's CommentSpan
            collected_spans.append(&mut instruction.0.collect_spans());
            // Add SemicolonToken's CommentSpan
            // TODO: probably we shouldn't allow for comments in between the instruction and comma since it may/will result in build failure after formatting
            collected_spans.push(CommentSpan::from_span(instruction.1.span()));
        }
        collected_spans
    }
}

impl CommentVisitor for Instruction {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Visit instructions as a whole unit, meaning we cannot insert comments inside an instruction.
        vec![CommentSpan::from_span(self.span())]
    }
}

impl CommentVisitor for AsmFinalExpr {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Add register's CommentSpan
        let mut collected_spans = vec![CommentSpan::from_span(self.register.span())];
        // Add ty's CommentSpan if it exists
        if let Some(ty) = &self.ty_opt {
            collected_spans.push(CommentSpan::from_span(ty.0.span()));
            // TODO: determine if we are allowing comments between `:` and ty
            collected_spans.append(&mut ty.1.collect_spans());
        }
        collected_spans
    }
}

impl CommentVisitor for IfExpr {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        // Add if_token's CommentSpan
        let mut collected_spans = vec![CommentSpan::from_span(self.if_token.span())];
        // Collect condition's CommentSpan
        collected_spans.append(&mut self.condition.collect_spans());
        // Collect then block
        collected_spans.append(&mut self.then_block.collect_spans());
        // Collect else if it exists
        if let Some(else_block) = &self.else_opt {
            // Add ElseToken's CommentSpan
            collected_spans.push(CommentSpan::from_span(else_block.0.span()));
            // Collect else & else if blocks' CommentSpans
            let mut else_body_spans = match &else_block.1 {
                std::ops::ControlFlow::Continue(if_expr) => if_expr.collect_spans(),
                std::ops::ControlFlow::Break(else_body) => else_body.collect_spans(),
            };
            collected_spans.append(&mut else_body_spans);
        }
        collected_spans
    }
}

impl CommentVisitor for IfCondition {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        match self {
            IfCondition::Expr(expr) => expr.collect_spans(),
            IfCondition::Let {
                let_token,
                lhs,
                eq_token,
                rhs,
            } => {
                let mut collected_spans = vec![CommentSpan::from_span(let_token.span())];
                collected_spans.append(&mut lhs.collect_spans());
                collected_spans.push(CommentSpan::from_span(eq_token.span()));
                collected_spans.append(&mut rhs.collect_spans());
                collected_spans
            }
        }
    }
}

impl CommentVisitor for MatchBranch {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        // Collect Pattern's CommentSpans
        collected_spans.append(&mut self.pattern.collect_spans());
        // Add fat_right_arrow_token's CommentSpan
        collected_spans.push(CommentSpan::from_span(self.fat_right_arrow_token.span()));
        // Collect kind's CommentSpans
        collected_spans.append(&mut self.kind.collect_spans());
        collected_spans
    }
}

impl CommentVisitor for MatchBranchKind {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        match self {
            MatchBranchKind::Block {
                block,
                comma_token_opt,
            } => {
                collected_spans.append(&mut block.collect_spans());
                // TODO: determine if we allow comments between block and comma_token
                if let Some(comma_token) = comma_token_opt {
                    collected_spans.push(CommentSpan::from_span(comma_token.span()));
                }
            }
            MatchBranchKind::Expr { expr, comma_token } => {
                collected_spans.append(&mut expr.collect_spans());
                // TODO: determine if we allow comments between expr and comma_token
                collected_spans.push(CommentSpan::from_span(comma_token.span()));
            }
        };
        collected_spans
    }
}

impl CommentVisitor for Assignable {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        match self {
            Assignable::Var(var) => collected_spans.push(CommentSpan::from_span(var.span())),
            Assignable::Index { target, arg } => {
                collected_spans.append(&mut target.collect_spans());
                collected_spans.append(&mut arg.collect_spans());
            }
            Assignable::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                collected_spans.append(&mut target.collect_spans());
                collected_spans.push(CommentSpan::from_span(dot_token.span()));
                collected_spans.push(CommentSpan::from_span(name.span()));
            }
            Assignable::TupleFieldProjection {
                target,
                dot_token,
                field: _field,
                field_span,
            } => {
                collected_spans.append(&mut target.collect_spans());
                collected_spans.push(CommentSpan::from_span(dot_token.span()));
                collected_spans.push(CommentSpan::from_span(field_span.clone()));
            }
        };
        collected_spans
    }
}
