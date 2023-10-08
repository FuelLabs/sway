use ast_node::expression::match_expression::typed::matcher::ReqDeclNode;
use either::Either;
use expression::typed_expression::{instantiate_lazy_and, instantiate_lazy_or};
use itertools::Itertools;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Spanned, Span};

use crate::{
    language::{parsed::MatchBranch, ty},
    semantic_analysis::*,
    types::DeterministicallyAborts,
    TypeInfo,
};

use super::matcher::matcher;

impl ty::TyMatchBranch {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        typed_value: &ty::TyExpression,
        branch: MatchBranch,
    ) -> Result<(ty::TyMatchBranch, ty::TyScrutinee), ErrorEmitted> {
        let MatchBranch {
            scrutinee,
            result,
            span: branch_span,
        } = branch;

        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        // type check the scrutinee
        let typed_scrutinee = ty::TyScrutinee::type_check(handler, ctx.by_ref(), scrutinee)?;

        // calculate the requirements and variable declarations
        let req_decl_tree = matcher(
            handler,
            ctx.by_ref(),
            typed_value, // This is the matched value. It gets propagated unchanged during matching for error reporting purposes.
            typed_value, // This is the same match value, but this time as the top level expression to be matched.
            typed_scrutinee.clone(),
        )?;

        // calculate if condition
        let if_condition = instantiate_if_condition(&req_decl_tree, handler, &mut ctx)?;

        // create a new namespace for this branch result
        let mut namespace = ctx.namespace.clone();
        let mut branch_ctx = ctx.scoped(&mut namespace);

        // TODO-IG: Replace with the declarations that take OR alternatives into account.
        // for every variable declaration, create a variable declaration,
        // insert it into the branch namespace, and add it to the block of code statements
        let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

        for (var_ident, var_decl_body) in req_decl_tree.variable_declarations().into_iter() {
            let type_ascription = var_decl_body.return_type.into();
            let return_type = var_decl_body.return_type;
            let span = var_ident.span().clone();
            let var_decl = ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
                name: var_ident.clone(),
                body: var_decl_body.clone(),
                mutability: ty::VariableMutability::Immutable,
                return_type,
                type_ascription,
            }));
            let _ = branch_ctx.insert_symbol(handler, var_ident.clone(), var_decl.clone());
            code_block_contents.push(ty::TyAstNode {
                content: ty::TyAstNodeContent::Declaration(var_decl),
                span,
            });
        }

        // type check the branch result
        let typed_result = {
            let ctx = branch_ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, result)?
        };

        // unify the return type from the typed result with the type annotation
        if !typed_result.deterministically_aborts(decl_engine, true) {
            branch_ctx.unify_with_type_annotation(handler, typed_result.return_type, &typed_result.span);
        }

        // if the typed branch result is a code block, then add the contents
        // of that code block to the block of code statements that we are already
        // generating. if the typed branch result is not a code block, then add
        // the typed branch result as an ast node to the block of code statements
        let ty::TyExpression {
            expression: typed_result_expression_variant,
            return_type: typed_result_return_type,
            span: typed_result_span,
        } = typed_result;
        match typed_result_expression_variant {
            ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock { mut contents, .. }) => {
                code_block_contents.append(&mut contents);
            }
            typed_result_expression_variant => {
                code_block_contents.push(ty::TyAstNode {
                    content: ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                        expression: typed_result_expression_variant,
                        return_type: typed_result_return_type,
                        span: typed_result_span.clone(),
                    }),
                    span: typed_result_span.clone(),
                });
            }
        }

        // assemble a new branch result that includes both the variable declarations
        // that we create and the typed result from the original untyped branch
        let new_result = ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                contents: code_block_contents,
            }),
            return_type: typed_result.return_type,
            span: typed_result_span,
        };

        let typed_branch = ty::TyMatchBranch {
            if_condition,
            result: new_result,
            span: branch_span,
        };

        Ok((typed_branch, typed_scrutinee)) // TODO-IG: Why not scrutinee part of the typed_branch?
    }
}

/// Returns a boolean expression that represents the total match arm requirement,
/// or `None` if the match arm is a catch-all arm.
/// E.g.: `struct.x == 11 && struct.y == 22 || struct.x == 33 && struct.y == 44`
fn instantiate_if_condition(req_decl_tree: &ReqDeclTree, handler: &Handler, ctx: &mut TypeCheckContext) -> Result<Option<ty::TyExpression>, ErrorEmitted> {
    return convert_req_decl_node_to_req_exp(handler, ctx, &req_decl_tree.root);

    fn convert_req_decl_node_to_req_exp(handler: &Handler, ctx: &mut TypeCheckContext, req_decl_node: &ReqDeclNode) -> Result<Option<ty::TyExpression>, ErrorEmitted> {
        return match req_decl_node {
            ReqDeclNode::ReqOrVarDecl(Some(Either::Left(req))) => instantiate_eq_requirement_expression(handler, ctx.by_ref(), &req.0, &req.1).map(|exp| Some(exp)),
            ReqDeclNode::ReqOrVarDecl(_) => Ok(None),
            ReqDeclNode::And(nodes) => {
                convert_and_or_or_req_decl_node(handler, ctx, nodes, |lhs, rhs| instantiate_lazy_and(ctx.engines, lhs, rhs))
            },
            ReqDeclNode::Or(nodes) => {
                convert_and_or_or_req_decl_node(handler, ctx, nodes, |lhs, rhs| instantiate_lazy_or(ctx.engines, lhs, rhs))
            },
        };

        fn convert_and_or_or_req_decl_node(handler: &Handler, ctx: &mut TypeCheckContext, nodes: &Vec<ReqDeclNode>, operator: impl Fn(ty::TyExpression, ty::TyExpression) -> ty::TyExpression) -> Result<Option<ty::TyExpression>, ErrorEmitted> {
            let req_nodes: Result<Vec<_>, _> = nodes.iter().map(|node| convert_req_decl_node_to_req_exp(handler, ctx, node)).collect();
            let req_nodes = req_nodes?.into_iter().filter_map(|node| node).collect_vec();
            match req_nodes[..] {
                [] => Ok(None),
                _ => Ok(Some(build_expression(&req_nodes[..], &operator))),
            }
        }
    
        fn build_expression(expressions: &[ty::TyExpression], operator: &impl Fn(ty::TyExpression, ty::TyExpression) -> ty::TyExpression) -> ty::TyExpression {
            let (lhs, others) = expressions.split_first().expect("The slice of requirement expressions must not be empty.");
            match others {
                [] => lhs.clone(),
                _ => operator(lhs.clone(), build_expression(others, operator)),
            }
        }
    }
}

/// Returns a boolean expression of the form `<lhs> == <rhs>`.
pub(crate) fn instantiate_eq_requirement_expression(handler: &Handler, ctx: TypeCheckContext, lhs: &ty::TyExpression, rhs: &ty::TyExpression) -> Result<ty::TyExpression, ErrorEmitted> {
    ty::TyExpression::core_ops_eq(
        handler,
        ctx,
        vec![lhs.clone(), rhs.clone()],
        Span::join(lhs.span.clone(), rhs.span.clone()),
    )
}
