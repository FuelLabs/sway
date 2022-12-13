use crate::core::{
    token::{to_ident_key, AstToken, SymbolKind, Token},
    token_map::TokenMap,
};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sway_ast::{
    Assignable, AttributeDecl, CodeBlockContents, Expr, ExprArrayDescriptor, ExprStructField,
    ExprTupleDescriptor, FnSignature, IfCondition, IfExpr, ItemKind, MatchBranchKind, Pattern,
    PatternStructField, Statement,
};
use sway_core::TypeEngine;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span, Spanned};

pub struct ParsedItems<'a> {
    tokens: &'a TokenMap,
}

impl<'a> ParsedItems<'a> {
    pub fn new(tokens: &'a TokenMap) -> Self {
        Self { tokens }
    }

    pub fn parse_module(&self, src: Arc<str>, path: Arc<PathBuf>) -> Result<(), ErrorEmitted> {
        let handler = <_>::default();
        let module = sway_parse::parse_file(&handler, src, Some(path.clone()))?;

        for item in module.items {
            item.value.parse(self.tokens);
        }
        Ok(())
    }
}

fn insert_keyword(tokens: &TokenMap, span: Span) {
    let ident = Ident::new(span);
    let token = Token::from_parsed(AstToken::Keyword(ident.clone()), SymbolKind::Keyword);
    tokens.insert(to_ident_key(&ident), token);
}

pub trait Parse {
    fn parse(&self, tokens: &TokenMap);
}

impl Parse for ItemKind {
    fn parse(&self, tokens: &TokenMap) {
        //eprintln!("item = {:#?}", self);
        match self {
            ItemKind::Fn(func) => {
                func.fn_signature.parse(tokens);
                func.body.get().parse(tokens);
            }
            ItemKind::Abi(abi) => for (_fn_sig, _) in &abi.abi_items.inner {},
            ItemKind::Impl(item_impl) => for _item_fn in &item_impl.contents.inner {},
            _ => (),
        }
    }
}

impl Parse for Expr {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Expr::AbiCast { abi_token, args } => {
                insert_keyword(tokens, abi_token.span());
                args.get().address.parse(tokens);
            }
            Expr::Struct { fields, .. } => {
                for expr in fields.get() {
                    expr.parse(tokens);
                }
            }
            Expr::Tuple(tuple) => {
                tuple.get().parse(tokens);
            }
            Expr::Parens(parens) => {
                parens.get().parse(tokens);
            }
            Expr::Block(block) => {
                block.get().parse(tokens);
            }
            Expr::Array(array) => {
                array.get().parse(tokens);
            }
            Expr::Return {
                return_token,
                expr_opt,
            } => {
                insert_keyword(tokens, return_token.span());
                if let Some(expr) = expr_opt {
                    expr.parse(tokens);
                }
            }
            Expr::If(if_expr) => {
                if_expr.parse(tokens);
            }
            Expr::Match {
                match_token,
                value,
                branches,
            } => {
                insert_keyword(tokens, match_token.span());
                value.parse(tokens);
                for branch in branches.get() {
                    branch.pattern.parse(tokens);
                    branch.kind.parse(tokens);
                }
            }
            Expr::While {
                while_token,
                condition,
                block,
            } => {
                insert_keyword(tokens, while_token.span());
                condition.parse(tokens);
                block.get().parse(tokens);
            }
            Expr::FuncApp { func, args } => {
                func.parse(tokens);
                for expr in args.get().into_iter() {
                    expr.parse(tokens);
                }
            }
            Expr::Index { target, arg } => {
                target.parse(tokens);
                arg.get().parse(tokens);
            }
            Expr::MethodCall {
                target,
                contract_args_opt,
                args,
                ..
            } => {
                target.parse(tokens);
                if let Some(contract_args) = contract_args_opt {
                    for expr in contract_args.get().into_iter() {
                        expr.parse(tokens);
                    }
                }
                for expr in args.get().into_iter() {
                    expr.parse(tokens);
                }
            }
            Expr::FieldProjection { target, .. } => {
                target.parse(tokens);
            }
            Expr::TupleFieldProjection { target, .. } => {
                target.parse(tokens);
            }
            Expr::Ref { ref_token, expr } => {
                insert_keyword(tokens, ref_token.span());
                expr.parse(tokens);
            }
            Expr::Deref { deref_token, expr } => {
                insert_keyword(tokens, deref_token.span());
                expr.parse(tokens);
            }
            Expr::Not { expr, .. } => {
                expr.parse(tokens);
            }
            Expr::Mul { lhs, rhs, .. }
            | Expr::Div { lhs, rhs, .. }
            | Expr::Pow { lhs, rhs, .. }
            | Expr::Modulo { lhs, rhs, .. }
            | Expr::Add { lhs, rhs, .. }
            | Expr::Sub { lhs, rhs, .. }
            | Expr::Shl { lhs, rhs, .. }
            | Expr::Shr { lhs, rhs, .. }
            | Expr::BitAnd { lhs, rhs, .. }
            | Expr::BitXor { lhs, rhs, .. }
            | Expr::BitOr { lhs, rhs, .. }
            | Expr::Equal { lhs, rhs, .. }
            | Expr::NotEqual { lhs, rhs, .. }
            | Expr::LessThan { lhs, rhs, .. }
            | Expr::GreaterThan { lhs, rhs, .. }
            | Expr::LessThanEq { lhs, rhs, .. }
            | Expr::GreaterThanEq { lhs, rhs, .. }
            | Expr::LogicalAnd { lhs, rhs, .. }
            | Expr::LogicalOr { lhs, rhs, .. } => {
                lhs.parse(tokens);
                rhs.parse(tokens);
            }
            Expr::Reassignment {
                assignable, expr, ..
            } => {
                assignable.parse(tokens);
                expr.parse(tokens);
            }
            Expr::Break { break_token } => {
                insert_keyword(tokens, break_token.span());
            }
            Expr::Continue { continue_token } => {
                insert_keyword(tokens, continue_token.span());
            }
            _ => {}
        }
    }
}

impl Parse for FnSignature {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.fn_token.span());
    }
}

impl Parse for CodeBlockContents {
    fn parse(&self, tokens: &TokenMap) {
        for statement in self.statements.iter() {
            statement.parse(tokens);
        }
    }
}

impl Parse for Statement {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Statement::Let(let_stmt) => {
                insert_keyword(tokens, let_stmt.let_token.span());
            }
            Statement::Expr { expr, .. } => {
                expr.parse(tokens);
            }
            Statement::Item(item) => {
                item.value.parse(tokens);
            }
        }
    }
}

impl Parse for ExprArrayDescriptor {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            ExprArrayDescriptor::Sequence(punct) => {
                for expr in punct.into_iter() {
                    expr.parse(tokens);
                }
            }
            ExprArrayDescriptor::Repeat { value, length, .. } => {
                value.parse(tokens);
                length.parse(tokens);
            }
        }
    }
}

impl Parse for IfExpr {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.if_token.span());
        self.condition.parse(tokens);
        self.then_block.get().parse(tokens);
        if let Some((else_token, control_flow)) = &self.else_opt {
            insert_keyword(tokens, else_token.span());
            match control_flow {
                ControlFlow::Break(block) => {
                    block.get().parse(tokens);
                }
                ControlFlow::Continue(if_expr) => {
                    if_expr.parse(tokens);
                }
            }
        }
    }
}

impl Parse for IfCondition {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            IfCondition::Expr(expr) => {
                expr.parse(tokens);
            }
            IfCondition::Let {
                let_token,
                lhs,
                rhs,
                ..
            } => {
                insert_keyword(tokens, let_token.span());
                lhs.parse(tokens);
                rhs.parse(tokens);
            }
        }
    }
}

impl Parse for Pattern {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Pattern::Var {
                reference, mutable, ..
            } => {
                if let Some(reference) = reference {
                    insert_keyword(tokens, reference.span());
                }
                if let Some(mutable) = mutable {
                    insert_keyword(tokens, mutable.span());
                }
            }
            Pattern::Constructor { args, .. } | Pattern::Tuple(args) => {
                for pattern in args.get().into_iter() {
                    pattern.parse(tokens);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields.get().into_iter() {
                    field.parse(tokens);
                }
            }
            _ => {}
        }
    }
}

impl Parse for PatternStructField {
    fn parse(&self, tokens: &TokenMap) {
        if let PatternStructField::Field { pattern_opt, .. } = self {
            if let Some((.., pattern)) = pattern_opt {
                pattern.parse(tokens);
            }
        }
    }
}

impl Parse for MatchBranchKind {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            MatchBranchKind::Block { block, .. } => {
                block.get().parse(tokens);
            }
            MatchBranchKind::Expr { expr, .. } => {
                expr.parse(tokens);
            }
        }
    }
}

impl Parse for ExprStructField {
    fn parse(&self, tokens: &TokenMap) {
        if let Some((.., expr)) = &self.expr_opt {
            expr.parse(tokens);
        }
    }
}

impl Parse for ExprTupleDescriptor {
    fn parse(&self, tokens: &TokenMap) {
        if let ExprTupleDescriptor::Cons { head, tail, .. } = self {
            head.parse(tokens);
            for expr in tail.into_iter() {
                expr.parse(tokens);
            }
        }
    }
}

impl Parse for Assignable {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Assignable::Index { target, arg } => {
                target.parse(tokens);
                arg.get().parse(tokens)
            }
            Assignable::FieldProjection { target, .. }
            | Assignable::TupleFieldProjection { target, .. } => {
                target.parse(tokens);
            }
            _ => {}
        }
    }
}
