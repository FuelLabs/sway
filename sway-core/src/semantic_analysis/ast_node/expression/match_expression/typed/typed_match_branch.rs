use std::convert::identity;

use ast_node::expression::match_expression::typed::matcher::ReqDeclNode;
use either::Either;
use itertools::Itertools;
use sway_error::{handler::{ErrorEmitted, Handler}, error::CompileError};
use sway_types::{Spanned, Span, Ident};

use crate::{
    language::{parsed::MatchBranch, ty::{self, MatchIfCondition, MatchMatchedOrVariantIndexVars}},
    semantic_analysis::*,
    types::DeterministicallyAborts,
    TypeInfo, TypeArgument, UnifyCheck, Engines, compiler_generated::{generate_matched_or_variant_index_var_name, INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL, generate_matched_or_variant_variables_var_name},
};

use super::{matcher::matcher, instantiate::Instantiate};

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

        // For the dummy span of all the instantiated code elements that cannot be mapped on
        // any of the elements from the original code, we will simply take the span of the
        // whole match arm. We assume that these spans will never be used.
        // This is also the error span in case of internal compiler errors.
        let instantiate = Instantiate::new(&ctx.engines, branch_span.clone());

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

        // Emit errors for eventual multiple definitions of variables.
        // We stop further compilation in case of duplicates in order to
        // provide guarantee to the desugaring that all the requirements
        // are satisfied for all of the variables:
        // - existence in all OR variants with the same type
        // - no duplicates
        handler.scope(|handler| {
            for duplicate in collect_duplicate_match_pattern_variables(&typed_scrutinee) {
                handler.emit_err(CompileError::MultipleDefinitionsOfMatchArmVariable {
                    match_value: typed_value.span.clone(),
                    match_type: engines.help_out(typed_value.return_type).to_string(),
                    first_definition: duplicate.first_definition.1,
                    first_definition_is_struct_field: duplicate.first_definition.0,
                    duplicate: duplicate.duplicate.1,
                    duplicate_is_struct_field: duplicate.duplicate.0,
                });
            }

            Ok(())
        })?;

        let (if_condition, result_var_declarations, or_variant_vars) = instantiate_if_condition_result_var_declarations_and_matched_or_variant_index_vars(&handler, &mut ctx, &instantiate, &req_decl_tree)?;

        // create a new namespace for this branch result
        let mut namespace = ctx.namespace.clone();
        let mut branch_ctx = ctx.scoped(&mut namespace);

        // for every variable that comes into result block, create a variable declaration,
        // insert it into the branch namespace, and add it to the block of code statements
        let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

        for (var_ident, var_body) in result_var_declarations {
            let var_decl = instantiate.var_decl(var_ident.clone(), var_body.clone());
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
    handler: &Handler,
    ctx: &mut TypeCheckContext,
    instantiate: &Instantiate,
    req_decl_tree: &ReqDeclTree
) -> Result<(MatchIfCondition, ResultVarDeclarations, MatchMatchedOrVariantIndexVars), ErrorEmitted> {
    let mut result_var_declarations = ResultVarDeclarations::new();
    let mut or_variants_vars = MatchMatchedOrVariantIndexVars::new();

    let result = instantiate_conditions_and_declarations(handler, ctx.by_ref(), &instantiate, None, &req_decl_tree.root, &mut result_var_declarations, &mut or_variants_vars)?;

    // At the end, there must not be any carry-over variable declarations.
    // All variable declarations must end up in the `result_var_declarations`.
    return if !result.1.is_empty() {
            Err(handler.emit_err(CompileError::Internal(
                "unable to extract match arm variables",
                instantiate.error_span(),
            )))
        }
        else {
            Ok((result.0, result_var_declarations, or_variants_vars))
        };

    fn instantiate_conditions_and_declarations(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        instantiate: &Instantiate,
        parent_node: Option<&ReqDeclNode>,
        req_decl_node: &ReqDeclNode,
        result_var_declarations: &mut ResultVarDeclarations,
        or_variants_vars: &mut MatchMatchedOrVariantIndexVars,
    ) -> Result<(MatchIfCondition, CarryOverVarDeclarations), ErrorEmitted> {
        return match req_decl_node {
            ReqDeclNode::ReqOrVarDecl(Some(Either::Left(req))) => {
                let condition = instantiate.eq_result(handler, ctx.by_ref(), req.0.clone(), req.1.clone()).map(|exp| Some(exp))?;
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
                instantiate_child_nodes_conditions_and_declarations(handler, ctx.by_ref(), &instantiate, &req_decl_node, parent_node.is_none(), nodes, result_var_declarations, or_variants_vars)
            },
        };

        fn instantiate_child_nodes_conditions_and_declarations(
            handler: &Handler,
            mut ctx: TypeCheckContext,
            instantiate: &Instantiate,
            parent_node: &ReqDeclNode,
            parent_node_is_root_node: bool,
            nodes: &Vec<ReqDeclNode>,
            result_var_declarations: &mut ResultVarDeclarations,
            matched_or_variant_index_vars: &mut MatchMatchedOrVariantIndexVars
        ) -> Result<(MatchIfCondition, CarryOverVarDeclarations), ErrorEmitted> {
            let conditions_and_carry_over_vars: Result<Vec<_>, _> = nodes.iter().map(|node| instantiate_conditions_and_declarations(handler, ctx.by_ref(), &instantiate, Some(parent_node), node, result_var_declarations, matched_or_variant_index_vars)).collect();
            let (conditions, carry_over_vars): (Vec<_>, Vec<_>) = conditions_and_carry_over_vars?.into_iter().unzip();

            let (condition, vars) = match parent_node {
                ReqDeclNode::And(_) => {
                    let conditions = conditions.into_iter().filter_map(identity).collect_vec();
                    let condition = match conditions[..] {
                        [] => None,
                        _ => Some(build_condition_expression(&conditions[..], &|lhs, rhs| instantiate.lazy_and(lhs, rhs))),
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
                        // Instantiate and return the expression for matched or variant index variable.
                        let suffix = matched_or_variant_index_vars.len() + 1;
                        let matched_or_variant_index_var_decl = instantiate_matched_or_variant_index_var_expression(&instantiate, suffix, conditions);
                        // Variable expression used to instantiate the corresponding tuple variable
                        // that will hold matched variant variables.
                        // Note that it is not needed to add the declaration of this variable
                        // to the context in order for the tuple variable to be created.
                        let matched_or_variant_index_variable = instantiate.var_exp(matched_or_variant_index_var_decl.0.clone(), matched_or_variant_index_var_decl.1.return_type);

                        matched_or_variant_index_vars.push(matched_or_variant_index_var_decl);

                        // Instantiate the tuple variable and the redefined variable declarations
                        // of the variables declared in OR variants.

                        let (tuple, mut redefined_vars) = instantiate_matched_or_variant_vars_expressions(
                            handler,
                            ctx.by_ref(),
                            &instantiate,
                            &matched_or_variant_index_variable,
                            suffix,
                            carry_over_vars)?;

                        // Always push the tuple declaration to the result variable declarations.
                        result_var_declarations.push(tuple);

                        if parent_node_is_root_node {
                            // We are within an OR root node. Add all the variable declarations to the result var declarations and
                            // return the calculated condition and no carry over vars.
                            result_var_declarations.append(&mut redefined_vars); // `redefined_vars` are empty after this.
                        }

                        // Instantiate the new condition that will be just the check if the 1-based matched variant index is different
                        // then zero.
                        let zero_u64_literal = instantiate.u64_literal(0);

                        let condition = instantiate.neq_result(handler, ctx.by_ref(), matched_or_variant_index_variable.clone(), zero_u64_literal.clone())?;

                        // Return the condition and either the empty `redefined_vars` if the parent is the root node, or carry over
                        // all the redefined variable declarations to the upper nodes.
                        (Some(condition), redefined_vars)
                    }
                    else {
                        let conditions = conditions.into_iter().filter_map(identity).collect_vec();
                        let condition = match conditions[..] {
                            [] => None,
                            _ => Some(build_condition_expression(&conditions[..], &|lhs, rhs| instantiate.lazy_or(lhs, rhs))),
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
        fn instantiate_matched_or_variant_index_var_expression(instantiate: &Instantiate, suffix: usize, conditions: Vec<MatchIfCondition>) -> (Ident, ty::TyExpression) {
            let ident = instantiate.ident(generate_matched_or_variant_index_var_name(suffix));

            // Build the expression bottom up by putting the previous if expression into
            // the else part of the current one.
            // Note that we do not have any optimizations like removals of `else` in case of `if true`.
            // Match expression optimizations will be done on IR side.
            let number_of_alternatives = conditions.len();

            let mut if_expr = instantiate.code_block_with_implicit_return_u64(0);
            for (rev_index, condition) in conditions.into_iter().rev().enumerate() {
                let condition = match condition {
                    Some(condition_exp) => condition_exp,
                    None => instantiate.boolean_literal(true),
                };

                if_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::IfExp {
                        condition: Box::new(condition),
                        then: Box::new(instantiate.code_block_with_implicit_return_u64((number_of_alternatives - rev_index).try_into().unwrap())),
                        r#else: Some(Box::new(if_expr)), // Put the previous if into else.
                    },
                    return_type: instantiate.u64_type(),
                    span: instantiate.dummy_span(),
                }
            };

            return (ident, if_expr);
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
            handler: &Handler,
            mut ctx: TypeCheckContext,
            instantiate: &Instantiate,
            matched_or_variant_index_variable: &ty::TyExpression,
            suffix: usize,
            mut var_declarations: Vec<CarryOverVarDeclarations>
        ) -> Result<(VarDecl, Vec<VarDecl>), ErrorEmitted> {
            let type_engine = ctx.engines.te();
            // At this point we have the guarantee that we have:
            // - exactly the same variables in each OR variant
            // - that variables of the same name are of the same type
            // - that we do not have duplicates in variable names inside of alternatives

            // Sort variables in all alternatives by name to get deterministic ordering in tuples.
            // Note that the var declarations in match patterns are mutually independent, thus,
            // we can shuffle their ordering.

            for vars_in_alternative in var_declarations.iter_mut() {
                vars_in_alternative.sort_by(|(a, _), (b, _)| a.cmp(b));
            }

            // Still, check the above guarantee and emit internal compiler errors if they are not satisfied.
            check_variables_guarantee(handler, ctx.engines, &var_declarations, instantiate.error_span())?;

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
            let mut if_expr = instantiate.code_block_with_implicit_return_revert(INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL);
            for (rev_index, vars) in var_declarations.into_iter().rev().enumerate() {
                let condition = instantiate_or_variant_has_matched_condition(ctx.by_ref(), &instantiate, matched_or_variant_index_variable,(number_of_alternatives - rev_index).try_into().unwrap());

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
                                            span: instantiate.dummy_span(),
                                        }),
                                        span: instantiate.dummy_span(),
                                    }],
                                }),
                                return_type: tuple_type,
                                span: instantiate.dummy_span(),
                            }
                        ),
                        r#else: Some(Box::new(if_expr)), // Put the previous if into else.
                    },
                    return_type: tuple_type,
                    span: instantiate.dummy_span(),
                }
            };

            let matched_or_variant_variables_tuple_ident = instantiate.ident(generate_matched_or_variant_variables_var_name(suffix));
            
            // For every variable in alternatives, redefined it by initializing it to the corresponding tuple field.
            let mut redefined_variables = vec![];

            // Variable expression used to emit tuple index access.
            // Note that it is not needed to add the tuple declaration to the
            // context in order for the index access expression to be created.
            let tuple_variable = instantiate.var_exp(matched_or_variant_variables_tuple_ident.clone(), tuple_type);

            for (index, variable) in variable_names.into_iter().enumerate() {
                redefined_variables.push((variable, instantiate.tuple_elem_access(ctx.engines, tuple_variable.clone(), index)));
            }

            return Ok(((matched_or_variant_variables_tuple_ident, if_expr), redefined_variables));

            /// Creates a boolean condition of the form `<matched_or_variant_index_variable> == <variant_index>`.
            /// `matched_or_variant_index_variable` is the corresponding variable of the name `__match_matched_or_variant_index_<suffix>`.
            fn instantiate_or_variant_has_matched_condition(ctx: TypeCheckContext, instantiate: &Instantiate, matched_or_variant_index_variable: &ty::TyExpression, variant_index: u64) -> ty::TyExpression {
                let variant_index_exp = instantiate.u64_literal(variant_index);
                instantiate.eq(ctx, matched_or_variant_index_variable.clone(), variant_index_exp)
            }

            fn check_variables_guarantee(handler: &Handler, engines: &Engines, sorted_var_declarations: &Vec<CarryOverVarDeclarations>, error_span: Span) -> Result<(), ErrorEmitted> {
                // Guarantees:
                // - exactly the same variables in each OR variant
                // - variables of the same name are of the same type
                // - we do not have duplicates in variable names inside of alternatives
                let (first_alternative_vars, other_alternatives_vars) = sorted_var_declarations.split_first().expect("Variable declarations must come from at least two OR alternatives.");

                if other_alternatives_vars.iter().any(|vars| vars.len() != first_alternative_vars.len()) {
                    return Err(handler.emit_err(CompileError::Internal(
                        "OR alternatives have different number of declared variables",
                        error_span,
                    )));
                }

                let equality = UnifyCheck::non_dynamic_equality(engines);

                for (index, (var_name, var_exp)) in first_alternative_vars.iter().enumerate() {
                    for other_vars in other_alternatives_vars.iter() {
                        let (other_var_name, other_var_exp) = &other_vars[index];

                        if var_name != other_var_name {
                            return Err(handler.emit_err(CompileError::Internal(
                                "OR alternatives have different variables declared",
                                error_span,
                            )));
                        }

                        if !equality.check(var_exp.return_type, other_var_exp.return_type) {
                            return Err(handler.emit_err(CompileError::Internal(
                                "variables of the same name in OR alternatives have different types",
                                error_span,
                            )));
                        }
                    }
                }

                // Check for duplicates.
                // At this point, we know already that we have the same variables in all alternatives
                // and that they are sorted by name.
                // Means, we need to check just the first alternative, and its immediate successor.
                for index in 0..first_alternative_vars.len()-1 {
                    if &first_alternative_vars[index].0 == &first_alternative_vars[index+1].0 {
                        return Err(handler.emit_err(CompileError::Internal(
                            "OR alternatives have duplicate variables",
                            error_span,
                        )));
                    }
                }

                Ok(())
            }
        }
    }
}