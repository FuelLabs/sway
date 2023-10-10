use std::convert::identity;

use ast_node::expression::match_expression::typed::matcher::ReqDeclNode;
use either::Either;
use expression::typed_expression::{instantiate_lazy_and, instantiate_lazy_or};
use itertools::Itertools;
use sway_error::{handler::{ErrorEmitted, Handler}, error::CompileError};
use sway_types::{Spanned, Span, Ident};

use crate::{
    language::{parsed::MatchBranch, ty::{self, MatchIfCondition, MatchOrVariantVars}},
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

        let (if_condition, result_var_declarations, or_variant_vars) = instantiate_if_condition_result_var_declarations_and_or_variant_vars(&branch_span, &req_decl_tree, handler, &mut ctx)?;

        // create a new namespace for this branch result
        let mut namespace = ctx.namespace.clone();
        let mut branch_ctx = ctx.scoped(&mut namespace);

        // for every variable declaration, create a variable declaration,
        // insert it into the branch namespace, and add it to the block of code statements
        let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

        for (var_ident, var_decl_body) in result_var_declarations {
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
            or_variant_vars,
            if_condition,
            result: new_result,
            span: branch_span,
        };

        Ok((typed_branch, typed_scrutinee))
    }
}

/// TODO-IG: Document in detail.
type ResultVarDeclarations = Vec<(Ident, ty::TyExpression)>;
type CarryOverVarDeclarations = Vec<(Ident, ty::TyExpression)>;

/// TODO-IG: Document in detail.
fn instantiate_if_condition_result_var_declarations_and_or_variant_vars(branch_span: &Span, req_decl_tree: &ReqDeclTree, handler: &Handler, ctx: &mut TypeCheckContext) -> Result<(MatchIfCondition, ResultVarDeclarations, MatchOrVariantVars), ErrorEmitted> {
    let mut result_var_declarations = ResultVarDeclarations::new();
    let mut or_variants_vars = MatchOrVariantVars::new();

    let result = instantiate_conditions_and_declarations(handler, ctx, None, &req_decl_tree.root, &mut result_var_declarations, &mut or_variants_vars)?;

    // At the end, there must not be any carry-over variable declarations.
    // All variable declarations must end up in the `result_var_declarations`.
    return if !result.1.is_empty() {
            Err(handler.emit_err(CompileError::Internal(
                "unable to extract match arm variables",
                branch_span.clone(),
            )))
        }
        else {
            Ok((result.0, result_var_declarations, or_variants_vars))
        };

    fn instantiate_conditions_and_declarations(handler: &Handler, ctx: &mut TypeCheckContext, parent_node: Option<&ReqDeclNode>, req_decl_node: &ReqDeclNode, result_var_declarations: &mut ResultVarDeclarations, or_variants_vars: &mut MatchOrVariantVars) -> Result<(MatchIfCondition, CarryOverVarDeclarations), ErrorEmitted> {
        return match req_decl_node {
            ReqDeclNode::ReqOrVarDecl(Some(Either::Left(req))) => {
                let condition = instantiate_eq_requirement_expression(handler, ctx.by_ref(), &req.0, &req.1).map(|exp| Some(exp))?;
                Ok((condition, vec![]))
            },
            ReqDeclNode::ReqOrVarDecl(Some(Either::Right(decl))) => {
                if parent_node.is_none() {
                    // I am the root/only node. Add my declaration to the result var declarations and pass no requirements and no carry over vars.
                    result_var_declarations.push(decl.clone());
                    Ok((None, vec![]))
                }
                else {
                    // I am embedded with an AND or OR node. The parent node needs to decide what to do with my variable declaration.
                    Ok((None, vec![decl.clone()]))
                }
            },
            ReqDeclNode::ReqOrVarDecl(None) => Ok((None, vec![])),
            ReqDeclNode::And(nodes) | ReqDeclNode::Or(nodes) => {
                instantiate_child_nodes_conditions_and_declarations(handler, ctx, &req_decl_node, parent_node.is_none(), nodes, result_var_declarations, or_variants_vars)
            },
        };

        fn instantiate_child_nodes_conditions_and_declarations(
            handler: &Handler,
            ctx: &mut TypeCheckContext,
            parent_node: &ReqDeclNode,
            parent_node_is_root_node: bool,
            nodes: &Vec<ReqDeclNode>,
            result_var_declarations: &mut ResultVarDeclarations,
            or_variants_vars: &mut MatchOrVariantVars) -> Result<(MatchIfCondition, CarryOverVarDeclarations), ErrorEmitted>
        {
            let conditions_and_carry_over_vars: Result<Vec<_>, _> = nodes.iter().map(|node| instantiate_conditions_and_declarations(handler, ctx, Some(parent_node), node, result_var_declarations, or_variants_vars)).collect();
            let (conditions, carry_over_vars): (Vec<_>, Vec<_>) = conditions_and_carry_over_vars?.into_iter().unzip();
            let conditions = conditions.into_iter().filter_map(identity).collect_vec();

            let (condition, vars) = match parent_node {
                ReqDeclNode::And(_) => {
                    let condition = match conditions[..] {
                        [] => None,
                        _ => Some(build_condition_expression(&conditions[..], &|lhs, rhs| instantiate_lazy_and(ctx.engines, lhs, rhs))),
                    };
                    let mut vars = carry_over_vars.into_iter().flatten().collect_vec();

                    if parent_node_is_root_node {
                        // We are within an AND root node. Add all the variable declarations to the result var declarations and
                        // return the calculated condition and no carry over vars.
                        result_var_declarations.append(&mut vars); // `vars` are empty after this.
                    }

                    // Return the condition and either the empty `vars` if the parent is the root node, or carry over
                    // all the variable declarations from all the child nodes.
                    (condition, vars)
                },
                ReqDeclNode::Or(_) => {
                    let has_var_decls = carry_over_vars.iter().any(|v| !v.is_empty());

                    if has_var_decls {
                        unimplemented!("Declaring variables in match arm OR alternatives is not implemented.")
                    }
                    else {
                        let condition = match conditions[..] {
                            [] => None,
                            _ => Some(build_condition_expression(&conditions[..], &|lhs, rhs| instantiate_lazy_or(ctx.engines, lhs, rhs))),
                        };

                        (condition, vec![])
                    }
                },
                _ => unreachable!("A parent node can only be an AND or an OR node."),
            };


            Ok((condition, vars))
        }
    
        fn build_condition_expression(expressions: &[ty::TyExpression], operator: &impl Fn(ty::TyExpression, ty::TyExpression) -> ty::TyExpression) -> ty::TyExpression {
            let (lhs, others) = expressions.split_first().expect("The slice of requirement expressions must not be empty.");
            match others {
                [] => lhs.clone(),
                _ => operator(lhs.clone(), build_condition_expression(others, operator)),
            }
        }

        /// Returns a boolean expression of the form `<lhs> == <rhs>`.
        fn instantiate_eq_requirement_expression(handler: &Handler, ctx: TypeCheckContext, lhs: &ty::TyExpression, rhs: &ty::TyExpression) -> Result<ty::TyExpression, ErrorEmitted> {
            ty::TyExpression::core_ops_eq(
                handler,
                ctx,
                vec![lhs.clone(), rhs.clone()],
                Span::join(lhs.span.clone(), rhs.span.clone()),
            )
        }
    }
}