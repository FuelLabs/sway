use std::convert::identity;

use ast_node::expression::match_expression::typed::matcher::ReqDeclNode;
use either::Either;
use expression::typed_expression::{instantiate_lazy_and, instantiate_lazy_or, instantiate_tuple_index_access};
use itertools::Itertools;
use sway_error::{handler::{ErrorEmitted, Handler}, error::CompileError};
use sway_types::{Spanned, Span, Ident, constants::{MATCH_MATCHED_OR_VARIANT_INDEX_VAR_NAME_PREFIX, MATCH_MATCHED_OR_VARIANT_VARIABLES_VAR_NAME_PREFIX, INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL}, integer_bits::IntegerBits};

use crate::{
    language::{parsed::MatchBranch, ty::{self, MatchIfCondition, MatchMatchedOrVariantIndexVars}, Literal},
    semantic_analysis::*,
    types::DeterministicallyAborts,
    TypeInfo, TypeArgument, Engines, TypeId,
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

        let (if_condition, result_var_declarations, or_variant_vars) = instantiate_if_condition_result_var_declarations_and_matched_or_variant_index_vars(&branch_span, &req_decl_tree, handler, &mut ctx)?;

        // create a new namespace for this branch result
        let mut namespace = ctx.namespace.clone();
        let mut branch_ctx = ctx.scoped(&mut namespace);

        // for every variable that comes into result block, create a variable declaration,
        // insert it into the branch namespace, and add it to the block of code statements
        let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

        for (var_ident, var_body) in result_var_declarations {
            let var_decl = instantiate_variable_declaration(&var_ident, &var_body);
            let _ = branch_ctx.insert_symbol(handler, var_ident.clone(), var_decl.clone());
            code_block_contents.push(ty::TyAstNode {
                content: ty::TyAstNodeContent::Declaration(var_decl),
                span: var_ident.span(),
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
            matched_or_variant_index_vars: or_variant_vars,
            if_condition,
            result: new_result,
            span: branch_span,
        };

        Ok((typed_branch, typed_scrutinee))
    }
}

type VarDecl = (Ident, ty::TyExpression);
/// Declarations of variables that have to be inserted at the beginning
/// of the match arm result.
type ResultVarDeclarations = Vec<VarDecl>;
/// Declarations of variables that are carried over from the lower parts
/// of the [ReqDeclTree] towards the upper parts. The decision which of
/// those variables should be added to [ResultVarDeclarations] is always
/// done at the AND and OR nodes upper in the tree.
type CarryOverVarDeclarations = Vec<VarDecl>;

/// TODO-IG: Document in detail.
fn instantiate_if_condition_result_var_declarations_and_matched_or_variant_index_vars(
    branch_span: &Span,
    req_decl_tree: &ReqDeclTree,
    handler: &Handler,
    ctx: &mut TypeCheckContext
) -> Result<(MatchIfCondition, ResultVarDeclarations, MatchMatchedOrVariantIndexVars), ErrorEmitted> {
    let mut result_var_declarations = ResultVarDeclarations::new();
    let mut or_variants_vars = MatchMatchedOrVariantIndexVars::new();

    // For the dummy span of all the instantiated code elements that cannot be mapped on
    // any of the elements from the original code, we will simply take the span of the
    // whole match arm. We assume that these spans will never be used.
    let result = instantiate_conditions_and_declarations(handler, ctx.by_ref(), None, &req_decl_tree.root, &mut result_var_declarations, &mut or_variants_vars, branch_span.clone())?;

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

    fn instantiate_conditions_and_declarations(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        parent_node: Option<&ReqDeclNode>,
        req_decl_node: &ReqDeclNode,
        result_var_declarations: &mut ResultVarDeclarations,
        or_variants_vars: &mut MatchMatchedOrVariantIndexVars,
        dummy_span: Span
    ) -> Result<(MatchIfCondition, CarryOverVarDeclarations), ErrorEmitted> {
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
                instantiate_child_nodes_conditions_and_declarations(handler, ctx.by_ref(), &req_decl_node, parent_node.is_none(), nodes, result_var_declarations, or_variants_vars, dummy_span)
            },
        };

        fn instantiate_child_nodes_conditions_and_declarations(
            handler: &Handler,
            mut ctx: TypeCheckContext,
            parent_node: &ReqDeclNode,
            parent_node_is_root_node: bool,
            nodes: &Vec<ReqDeclNode>,
            result_var_declarations: &mut ResultVarDeclarations,
            matched_or_variant_index_vars: &mut MatchMatchedOrVariantIndexVars,
            dummy_span: Span
        ) -> Result<(MatchIfCondition, CarryOverVarDeclarations), ErrorEmitted> {
            let conditions_and_carry_over_vars: Result<Vec<_>, _> = nodes.iter().map(|node| instantiate_conditions_and_declarations(handler, ctx.by_ref(), Some(parent_node), node, result_var_declarations, matched_or_variant_index_vars, dummy_span.clone())).collect();
            let (conditions, carry_over_vars): (Vec<_>, Vec<_>) = conditions_and_carry_over_vars?.into_iter().unzip();

            let (condition, vars) = match parent_node {
                ReqDeclNode::And(_) => {
                    let conditions = conditions.into_iter().filter_map(identity).collect_vec();
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
                    // We instantiate plenty of these. To avoid passing u64 return type or engines, dummy span, etc.
                    // we will use and pass this closure.
                    let instantiate_u64_literal =  {
                        let type_engine = ctx.engines.te();
                        let return_type = type_engine.insert(ctx.engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
                        let dummy_span = dummy_span.clone();

                        move |value: u64| {
                            ty::TyExpression {
                                expression: ty::TyExpressionVariant::Literal(Literal::U64(value)),
                                return_type,
                                span: dummy_span.clone(),
                            }
                        }
                    };

                    let has_var_decls = carry_over_vars.iter().any(|v| !v.is_empty());

                    if has_var_decls {
                        // Instantiate and return the expression for matched or variant index variable.
                        let suffix = matched_or_variant_index_vars.len() + 1;
                        let matched_or_variant_index_var_decl = instantiate_matched_or_variant_index_var_expression(ctx.engines, suffix, dummy_span.clone(), conditions);
                        // Variable expression used to instantiate the corresponding tuple variable
                        // that will hold matched variant variables.
                        // Note that it is not needed to add the declaration of this variable
                        // to the context in order for the tuple variable to be created.
                        let matched_or_variant_index_variable = instantiate_variable_expression(matched_or_variant_index_var_decl.0.clone(), matched_or_variant_index_var_decl.1.return_type, dummy_span.clone());

                        matched_or_variant_index_vars.push(matched_or_variant_index_var_decl);

                        // Instantiate the tuple variable and the redefined variable declarations
                        // of the variables declared in OR variants.

                        let (tuple, mut redefined_vars) = instantiate_matched_or_variant_vars_expressions(
                            ctx.by_ref(),
                            &matched_or_variant_index_variable,
                            suffix,
                            carry_over_vars,
                            dummy_span.clone());

                        // Always push the tuple declaration to the result variable declarations.
                        result_var_declarations.push(tuple);

                        if parent_node_is_root_node {
                            // We are within an OR root node. Add all the variable declarations to the result var declarations and
                            // return the calculated condition and no carry over vars.
                            result_var_declarations.append(&mut redefined_vars); // `redefined_vars` are empty after this.
                        }

                        // Instantiate the new condition that will be just the check if the 1-based matched variant index is different
                        // then zero.
                        let zero_u64_literal = instantiate_u64_literal(0);

                        let condition = instantiate_neq_requirement_expression(handler, ctx.by_ref(), &matched_or_variant_index_variable, &zero_u64_literal)?;

                        // Return the condition and either the empty `redefined_vars` if the parent is the root node, or carry over
                        // all the redefined variable declarations to the upper nodes.
                        (Some(condition), redefined_vars)
                    }
                    else {
                        let conditions = conditions.into_iter().filter_map(identity).collect_vec();
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

        /// Returns a boolean expression of the form `<lhs> != <rhs>`.
        fn instantiate_neq_requirement_expression(handler: &Handler, ctx: TypeCheckContext, lhs: &ty::TyExpression, rhs: &ty::TyExpression) -> Result<ty::TyExpression, ErrorEmitted> {
            ty::TyExpression::core_ops_neq(
                handler,
                ctx,
                vec![lhs.clone(), rhs.clone()],
                Span::join(lhs.span.clone(), rhs.span.clone()),
            )
        }

        /// Instantiates an immutable variable declaration for the variable
        /// that tracks which of the OR variants got matched, if any.
        /// If one of the variants match, the variable will be initialized
        /// to the 1-based index of that variant counted from left to right.
        /// If none of the variants match the variable will be initialized
        /// to zero.
        /// 
        /// let __match_matched_or_variant_index_<suffix>: u64 = if <variant_1_condition> {
        ///         1u64
        ///     } else if <variant_2_condition> {
        ///         2u64
        ///     } else if ... {
        ///         ...
        ///     }
        ///     } else {
        ///         0u64
        ///     };
        fn instantiate_matched_or_variant_index_var_expression(engines: &Engines, suffix: usize, dummy_span: Span, conditions: Vec<MatchIfCondition>) -> (Ident, ty::TyExpression) {
            debug_assert!(suffix > 0, "The per match arm unique suffix must be grater than zero.");

            let type_engine = engines.te();

            let var_name = format!("{}{}", MATCH_MATCHED_OR_VARIANT_INDEX_VAR_NAME_PREFIX, suffix);
            let ident = Ident::new_with_override(var_name, dummy_span.clone());

            // Build the `if-else` chain bottom up, means traverse in reverse order.

            // TODO-IG: Optimize if some of conditions is None. Check if current match desugaring is optimized for this case.
            // Build the expression bottom up by putting the previous if expression into
            // the else part of the current one.
            let return_type = type_engine.insert(engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
            let number_of_alternatives = conditions.len();

            let mut if_expr = instantiate_final_else_block(return_type, dummy_span.clone());
            for (rev_index, condition) in conditions.into_iter().rev().enumerate() {
                let condition = match condition {
                    Some(condition_exp) => condition_exp,
                    None => ty::TyExpression {
                                expression: ty::TyExpressionVariant::Literal(Literal::Boolean(true)),
                                return_type: type_engine.insert(engines, TypeInfo::Boolean),
                                span: dummy_span.clone(),
                            }
                };

                if_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::IfExp {
                        condition: Box::new(condition),
                        then: Box::new(
                            ty::TyExpression {
                                expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                                    contents: vec![ty::TyAstNode {
                                        content: ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                                            expression: ty::TyExpressionVariant::Literal(Literal::U64((number_of_alternatives - rev_index).try_into().unwrap())),
                                            return_type,
                                            span: dummy_span.clone(),
                                        }),
                                        span: dummy_span.clone(),
                                    }],
                                }),
                                return_type,
                                span: dummy_span.clone(),
                            }
                        ),
                        r#else: Some(Box::new(if_expr)), // Put the previous if into else.
                    },
                    return_type,
                    span: dummy_span.clone(),
                }
            };

            return (ident, if_expr);

            /// Instantiates a block with an implicit return of `0u64`.
            fn instantiate_final_else_block(return_type: TypeId, dummy_span: Span) -> ty::TyExpression {
                ty::TyExpression {
                    expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                        contents: vec![ty::TyAstNode {
                            content: ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                                expression: ty::TyExpressionVariant::Literal(Literal::U64(0)),
                                return_type,
                                span: dummy_span.clone(),
                            }),
                            span: dummy_span.clone(),
                        }],
                    }),
                    return_type,
                    span: dummy_span,
                }
            }
        }

        /// Instantiates immutable variable declarations for all the variables
        /// declared in an OR match expression.
        /// Choosing the right initialization, the initialization coming from
        /// the OR variant that actually matched, is done by inspecting
        /// the result of the corresponding __match_matched_or_variant_index_<suffix>
        /// variable.
        /// 
        /// The function returns:
        /// - a variable declaration of a temporary tuple variable that holds
        ///   the values of all the variables declared in the OR match expression
        /// - declarations of each individual variable.
        /// 
        /// let __match_matched_or_variant_variables_<suffix>: <tuple> = if __match_matched_or_variant_index_<suffix> == 1 {
        ///         (<var_1_variant_1_initialization>, ... <var_n_variant_1_initialization>)
        ///     } else if __match_matched_or_variant_index_<suffix> == 2 {
        ///         (<var_1_variant_2_initialization>, ... <var_n_variant_2_initialization>)
        ///     } else if ... {
        ///         ...
        ///     }
        ///     } else {
        ///         __revert(...) // This should never happen and means internal compiler error.
        ///     };
        /// 
        /// let <var_1> = __match_matched_or_variant_variables_<suffix>.0;
        /// let <var_2> = __match_matched_or_variant_variables_<suffix>.1;
        /// ...
        /// let <var_n> = __match_matched_or_variant_variables_<suffix>.(n-1);
        fn instantiate_matched_or_variant_vars_expressions(
            mut ctx: TypeCheckContext,
            matched_or_variant_index_variable: &ty::TyExpression,
            suffix: usize,
            mut var_declarations: Vec<CarryOverVarDeclarations>,
            dummy_span: Span
        ) -> (VarDecl, Vec<VarDecl>) {
            let type_engine = ctx.engines.te();
            // TODO-IG: Assert invariants.
            // At this point we have the guarantee given by the matcher that we have:
            // - exactly the same variables in each OR variant
            // - that variables of the same name are of the same type
            // - that we do not have duplicates in variable names inside of alternatives

            // Sort variables in all alternatives by name to get deterministic ordering in tuples.
            // Note that the var declarations in match patterns are mutually independent, thus,
            // we can shuffle their ordering.

            for vars_in_alternative in var_declarations.iter_mut() {
                vars_in_alternative.sort_by(|(a, _), (b, _)| a.cmp(b));
            }

            // Build the `if-else` chain for the declaration of the tuple variable.
            // Build it bottom up, means traverse in reverse order.

            // All variants have same variable types and names, thus we pick them from the first alternative.
            let tuple_field_types = var_declarations[0].iter().map(|(_, var_body)| TypeArgument {
                type_id: var_body.return_type,
                initial_type_id: var_body.return_type,
                span: var_body.span.clone(), // Although not needed, this span can be mapped to var declaration.
                call_path_tree: None,
            }).collect();
            let tuple_type = type_engine.insert(ctx.engines, TypeInfo::Tuple(tuple_field_types));
            let variable_names = var_declarations[0].iter().map(|(ident, _)| ident.clone()).collect_vec();

            // Build the expression bottom up by putting the previous if expression into
            // the else part of the current one.
            let number_of_alternatives = var_declarations.len();
            let mut if_expr = instantiate_final_else_block(ctx.engines, dummy_span.clone());
            for (rev_index, vars) in var_declarations.into_iter().rev().enumerate() {
                let condition = instantiate_or_variant_has_matched_condition(ctx.by_ref(), matched_or_variant_index_variable,(number_of_alternatives - rev_index).try_into().unwrap(), dummy_span.clone());

                if_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::IfExp {
                        condition: Box::new(condition),
                        then: Box::new(
                            ty::TyExpression {
                                expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                                    contents: vec![ty::TyAstNode {
                                        content: ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                                            expression: ty::TyExpressionVariant::Tuple {
                                                fields: vars.into_iter().map(|(_, exp)| exp).collect(),
                                            },
                                            return_type: tuple_type,
                                            span: dummy_span.clone(),
                                        }),
                                        span: dummy_span.clone(),
                                    }],
                                }),
                                return_type: tuple_type,
                                span: dummy_span.clone(),
                            }
                        ),
                        r#else: Some(Box::new(if_expr)), // Put the previous if into else.
                    },
                    return_type: tuple_type,
                    span: dummy_span.clone(),
                }
            };

            let matched_or_variant_variables_tuple_ident = Ident::new_with_override(format!("{}{}", MATCH_MATCHED_OR_VARIANT_VARIABLES_VAR_NAME_PREFIX, suffix), dummy_span.clone());
            
            // For every variable in alternatives, redefined it by initializing it to the corresponding tuple field.
            let mut redefined_variables = vec![];

            // Variable expression used to emit tuple index access.
            // Note that it is not needed to add the tuple declaration to the
            // context in order for the index access expression to be created.
            let tuple_variable = instantiate_variable_expression(matched_or_variant_variables_tuple_ident.clone(), tuple_type, dummy_span.clone());

            for (index, variable) in variable_names.into_iter().enumerate() {
                let var_body = instantiate_tuple_index_access(&Handler::default(), ctx.engines, tuple_variable.clone(), index, dummy_span.clone(), dummy_span.clone())
                    .ok()
                    .expect("Creating tuple index access expression for matched OR variants must always work.");

                redefined_variables.push((variable, var_body));
            }

            return ((matched_or_variant_variables_tuple_ident, if_expr), redefined_variables);

            /// Creates a boolean condition of the form `<matched_or_variant_index_variable> == <variant_index>`.
            /// `matched_or_variant_index_variable` is the corresponding variable of the name `__match_matched_or_variant_index_<suffix>`.
            fn instantiate_or_variant_has_matched_condition(ctx: TypeCheckContext, matched_or_variant_index_variable: &ty::TyExpression, variant_index: u64, dummy_span: Span) -> ty::TyExpression {
                let type_engine = ctx.engines.te();

                let variant_index_exp = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Literal(Literal::U64(variant_index)),
                    return_type: type_engine.insert(ctx.engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
                    span: dummy_span.clone(),
                };

                ty::TyExpression::core_ops_eq(&Handler::default(), ctx, vec![matched_or_variant_index_variable.clone(), variant_index_exp], dummy_span)
                    .ok()
                    .expect("Comparing two `u64` values must always work.")
            }

            /// Instantiates a block with an implicit return of `__revert()`.
            fn instantiate_final_else_block(engines: &Engines, dummy_span: Span) -> ty::TyExpression {
                let type_engine = engines.te();
                let revert_type = type_engine.insert(engines, TypeInfo::Unknown); // TODO: Change this to the `Never` type once available.

                ty::TyExpression {
                    expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                        contents: vec![ty::TyAstNode {
                            content: ty::TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                                    expression: ty::TyExpressionVariant::IntrinsicFunction(ty::TyIntrinsicFunctionKind {
                                        kind: sway_ast::Intrinsic::Revert,
                                        arguments: vec![ty::TyExpression {
                                            expression: ty::TyExpressionVariant::Literal(Literal::U64(INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL)),
                                            return_type: type_engine.insert(engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
                                            span: dummy_span.clone(),
                                        }],
                                        type_arguments: vec![],
                                        span: dummy_span.clone(),
                                    }),
                                    return_type: revert_type,
                                    span: dummy_span.clone(),
                                }),
                            span: dummy_span.clone(),
                        }],
                    }),
                    return_type: revert_type,
                    span: dummy_span,
                }
            }
        }
    }
}

fn instantiate_variable_expression(ident: Ident, type_id: TypeId, dummy_span: Span) -> ty::TyExpression {
    ty::TyExpression {
        expression: ty::TyExpressionVariant::VariableExpression {
            name: ident,
            span: dummy_span.clone(),
            mutability: ty::VariableMutability::Immutable,
            call_path: None
        },
        return_type: type_id,
        span: dummy_span,
    }
}

/// Instantiates an immutable variable declaration of the form `let <ident> = <body>`.
fn instantiate_variable_declaration(ident: &Ident, body: &ty::TyExpression) -> ty::TyDecl {
    let type_ascription = body.return_type.into();
    let return_type = body.return_type;
    let var_decl = ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
        name: ident.clone(),
        body: body.clone(),
        mutability: ty::VariableMutability::Immutable,
        return_type,
        type_ascription,
    }));

    var_decl
}