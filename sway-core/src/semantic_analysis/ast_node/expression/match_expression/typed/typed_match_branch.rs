use ast_node::expression::match_expression::typed::matcher::{ReqDeclNode, ReqOrVarDecl};
use indexmap::IndexSet;
use itertools::{multiunzip, Itertools};
use sway_error::{
    error::{CompileError, ShadowingSource},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    compiler_generated::{
        generate_matched_or_variant_index_var_name, generate_matched_or_variant_variables_var_name,
        INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL,
    },
    language::{
        parsed::MatchBranch,
        ty::{self, MatchBranchCondition, MatchedOrVariantIndexVars, TyExpression},
    },
    semantic_analysis::*,
    Engines, TypeInfo, UnifyCheck,
};

use super::{instantiate::Instantiate, matcher::matcher, ReqDeclTree};

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
        let instantiate = Instantiate::new(ctx.engines, branch_span.clone());

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // Type check the scrutinee.
        let typed_scrutinee = ty::TyScrutinee::type_check(handler, ctx.by_ref(), scrutinee)?;

        // Calculate the requirements and variable declarations.
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

        // Emit errors for eventual uses of configurables in patterns.
        // Configurables cannot be used in pattern matching, since they are not compile-time
        // constants. Using a configurable will define a pattern variable, that will
        // then shadow the configurable of the same name, which is not allowed.
        // This can be very confusing in case someone tries to use configurables in pattern matching
        // in the same way as constants. So we provide helpful hints here.
        // We stop further compilation in case of finding configurables in patterns.
        handler.scope(|handler| {
            // All the first occurrences of variables in order of appearance, while respecting
            // if they are struct field variables.
            let variables: IndexSet<(Ident, bool)> =
                IndexSet::from_iter(collect_match_pattern_variables(&typed_scrutinee));
            for (ident, is_struct_field) in variables {
                let default_handler = &Handler::default();
                // If there exist a configurable with the same name as the pattern variable.
                if let Ok(ty::TyDecl::ConfigurableDecl(configurable_decl)) =
                    ctx.resolve_symbol(default_handler, &ident)
                {
                    let name = (&ident).into();
                    let configurable_span = engines
                        .de()
                        .get_configurable(&configurable_decl.decl_id)
                        .span();
                    if is_struct_field {
                        handler.emit_err(CompileError::ConfigurablesCannotBeShadowed {
                            shadowing_source: ShadowingSource::PatternMatchingStructFieldVar,
                            name,
                            configurable_span,
                        });
                    } else {
                        handler.emit_err(CompileError::ConfigurablesCannotBeMatchedAgainst {
                            name,
                            configurable_span,
                        });
                    }
                }
            }

            Ok(())
        })?;

        let (condition, result_var_declarations, or_variant_vars) =
            instantiate_branch_condition_result_var_declarations_and_matched_or_variant_index_vars(
                handler,
                &mut ctx,
                &instantiate,
                &req_decl_tree,
            )?;

        // create a new namespace for this branch result
        ctx.scoped(handler, Some(branch_span.clone()), |scoped_ctx| {
            // for every variable that comes into result block, create a variable declaration,
            // insert it into the branch namespace, and add it to the block of code statements
            let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

            for (var_ident, var_body) in result_var_declarations {
                let var_decl = instantiate.var_decl(var_ident.clone(), var_body.clone());
                let _ = scoped_ctx.insert_symbol(handler, var_ident.clone(), var_decl.clone());
                code_block_contents.push(ty::TyAstNode {
                    content: ty::TyAstNodeContent::Declaration(var_decl),
                    span: var_ident.span(),
                });
            }

            // type check the branch result
            let typed_result = {
                // If there is an expectation coming from the context via `ctx.type_annotation()` we need
                // to pass that contextual requirement to the branch in order to provide more specific contextual
                // information. E.g., that `Option<u8>` is expected.
                // But at the same time, we do not want to unify during type checking with that contextual information
                // at this stage, because the branch might get `TypeInfo::Unknown` as the expectation and diverge
                // at the same time. The divergence would unify `TypeInfo::Never` and `Unknown` in that case, leaving
                // `Never` as the expected type for the subsequent branches.
                // In order to pass the contextual information, but not to affect the original type with potential
                // unwanted unification with `Never`, we create a copies of the `ctx.type_annotation()` type and pass
                // it as the expectation to the branch.
                let type_annotation = (*type_engine.get(scoped_ctx.type_annotation())).clone();
                let branch_ctx = scoped_ctx.by_ref().with_type_annotation(type_engine.insert(
                    engines,
                    type_annotation,
                    None,
                ));
                ty::TyExpression::type_check(handler, branch_ctx, &result)?
            };

            // Check if return type is Never if it is we don't unify as it would replace the Unknown annotation with Never.
            if !matches!(*type_engine.get(typed_result.return_type), TypeInfo::Never) {
                // unify the return type from the typed result with the type annotation
                // Note here that the `scoped_ctx` is actually the original `ctx` just scoped
                // to the `namespace`, thus, having the same original type annotation.
                // This unification is also the mechanism for carrying the type of a branch to
                // the subsequent branch. It potentially alters the type behind the `ctx.type_annotation()`
                // which will then be picked by the next branch.
                scoped_ctx.unify_with_type_annotation(
                    handler,
                    typed_result.return_type,
                    &typed_result.span,
                );
            }

            // if the typed branch result is a code block, then add the contents
            // of that code block to the block of code statements that we are already
            // generating. if the typed branch result is not a code block, then add
            // the typed branch result as an ast node to the block of code statements
            let typed_result_return_type = typed_result.return_type;
            let typed_result_span = typed_result.span.clone();
            match typed_result.expression {
                ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock { mut contents, .. }) => {
                    code_block_contents.append(&mut contents);
                }
                _ => {
                    code_block_contents.push(ty::TyAstNode {
                        content: ty::TyAstNodeContent::Expression(TyExpression {
                            return_type: typed_result_return_type,
                            span: typed_result_span.clone(),
                            expression: ty::TyExpressionVariant::ImplicitReturn(Box::new(
                                typed_result,
                            )),
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
                    whole_block_span: sway_types::Span::dummy(),
                }),
                return_type: typed_result_return_type,
                span: typed_result_span,
            };

            let typed_branch = ty::TyMatchBranch {
                matched_or_variant_index_vars: or_variant_vars,
                condition,
                result: new_result,
                span: branch_span,
            };

            Ok((typed_branch, typed_scrutinee))
        })
    }
}

type VarDecl = (Ident, ty::TyExpression);
/// Declarations of variables that have to be inserted at the beginning
/// of the match arm result.
/// These can be simple variable declarations in the form `let <ident> = <exp>;`
/// or the declarations of tuple variables holding values coming from OR
/// variants. In the former case, the variable body can be an arbitrary long
/// chain of nested `if` expressions: `let <tuple> = if .. else ..`.
type ResultVarDeclarations = Vec<VarDecl>;
/// Declarations of variables that are carried over from the lower parts
/// of the [ReqDeclTree] towards the upper parts. The decision which of
/// those variables should be added to [ResultVarDeclarations] is always
/// done at the AND and OR nodes upper in the tree.
/// The OR nodes can transform the variables before passing them to the
/// upper nodes.
type CarryOverVarDeclarations = Vec<VarDecl>;
/// Declarations of tuple variables that are carried over from the lower parts
/// of the [ReqDeclTree] towards the upper parts. The decision which of
/// those tuple variables should be added to [ResultVarDeclarations] is always
/// done at the AND and OR nodes upper in the tree.
/// The OR nodes can embed tuple variables into definitions of other tuple
/// variables, thus, not passing them any more to the upper nodes.
type CarryOverTupleDeclarations = Vec<VarDecl>;

/// Instantiates three artifacts, that are in the end carried over to the typed match expression
/// via [ty::TyMatchBranch]:
/// - branch condition: Overall condition that must be `true` for the branch to match.
/// - result variable declarations: Variable declarations that needs to be added to the
///   match branch result, before the actual body. Here we distinguish between the variables
///   actually declared in the match arm pattern and so called "tuple variables" that are
///   compiler generated and contain values for variables extracted out of individual OR variants.
/// - OR variant index variables: Variable declarations that are generated in case of having
///   variables in OR patterns. Index variables hold 1-based index of the OR variant being matched
///   or zero if non of the OR variants has matched.
///
/// ## Algorithm Overview
/// The algorithm traverses the `req_decl_tree` bottom up from left to right and collects the
/// overall condition, variable declarations, and tuple variable declarations.
///
/// In general, if the visited node is not the root node, the variables and requirements encountered
/// at that node must be carried over to the upper node that decides how to interpret them.
///
/// E.g., if the upper node is an AND node with three sub nodes each having a requirement, the AND
/// node will decide to combine the three requirements using the lazy and operator, and to pass only
/// the new single requirement to the upper nodes.
///
/// Detailed explanation on how the condition and carry over declarations are constructed and
/// carried over is given on other implementation functions.
///
/// Examples of resulting desugared match expressions can be found in the module description ([super]);
fn instantiate_branch_condition_result_var_declarations_and_matched_or_variant_index_vars(
    handler: &Handler,
    ctx: &mut TypeCheckContext,
    instantiate: &Instantiate,
    req_decl_tree: &ReqDeclTree,
) -> Result<
    (
        MatchBranchCondition,
        ResultVarDeclarations,
        MatchedOrVariantIndexVars,
    ),
    ErrorEmitted,
> {
    let mut result_var_declarations = ResultVarDeclarations::new();
    let mut or_variants_index_vars = MatchedOrVariantIndexVars::new();

    let (condition, carry_over_var_declarations, carry_over_tuple_declarations) =
        recursively_instantiate_conditions_declarations_and_variant_index_vars(
            handler,
            ctx.by_ref(),
            instantiate,
            None,
            req_decl_tree.root(),
            &mut result_var_declarations,
            &mut or_variants_index_vars,
        )?;

    // At the end, there must not be any carry-over declarations.
    // All variable declarations must end up in the `result_var_declarations`.
    return if !(carry_over_var_declarations.is_empty() && carry_over_tuple_declarations.is_empty())
    {
        Err(handler.emit_err(CompileError::Internal(
            "unable to extract match arm variables",
            instantiate.error_span(),
        )))
    } else {
        Ok((condition, result_var_declarations, or_variants_index_vars))
    };

    fn recursively_instantiate_conditions_declarations_and_variant_index_vars(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        instantiate: &Instantiate,
        parent_node: Option<&ReqDeclNode>,
        req_decl_node: &ReqDeclNode,
        result_var_declarations: &mut ResultVarDeclarations,
        or_variants_index_vars: &mut MatchedOrVariantIndexVars,
    ) -> Result<
        (
            MatchBranchCondition,
            CarryOverVarDeclarations,
            CarryOverTupleDeclarations,
        ),
        ErrorEmitted,
    > {
        return match req_decl_node {
            ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::Req(req)) => {
                let condition = instantiate
                    .eq_result(handler, ctx.by_ref(), req.0.clone(), req.1.clone())
                    .map(Some)?;
                Ok((condition, vec![], vec![]))
            }
            ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::VarDecl(decl)) => {
                if parent_node.is_none() {
                    // I am the root/only node. Add my declaration to the result var declarations and pass no requirements and no carry over vars.
                    result_var_declarations.push(decl.clone());
                    Ok((None, vec![], vec![]))
                } else {
                    // I am embedded with an AND or OR node. The parent node needs to decide what to do with my variable declaration.
                    Ok((None, vec![decl.clone()], vec![]))
                }
            }
            ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::Neither) => Ok((None, vec![], vec![])),
            ReqDeclNode::And(nodes) | ReqDeclNode::Or(nodes) => {
                instantiate_child_nodes_conditions_and_declarations(
                    handler,
                    ctx.by_ref(),
                    instantiate,
                    req_decl_node,
                    parent_node.is_none(),
                    nodes,
                    result_var_declarations,
                    or_variants_index_vars,
                )
            }
        };

        #[allow(clippy::too_many_arguments)]
        fn instantiate_child_nodes_conditions_and_declarations(
            handler: &Handler,
            mut ctx: TypeCheckContext,
            instantiate: &Instantiate,
            parent_node: &ReqDeclNode,
            parent_node_is_root_node: bool,
            nodes: &[ReqDeclNode],
            result_var_declarations: &mut ResultVarDeclarations,
            or_variant_index_vars: &mut MatchedOrVariantIndexVars,
        ) -> Result<
            (
                MatchBranchCondition,
                CarryOverVarDeclarations,
                CarryOverTupleDeclarations,
            ),
            ErrorEmitted,
        > {
            let conditions_and_carry_overs: Result<Vec<_>, _> = nodes
                .iter()
                .map(|node| {
                    recursively_instantiate_conditions_declarations_and_variant_index_vars(
                        handler,
                        ctx.by_ref(),
                        instantiate,
                        Some(parent_node),
                        node,
                        result_var_declarations,
                        or_variant_index_vars,
                    )
                })
                .collect();
            let (conditions, carry_over_vars, carry_over_tuples): (Vec<_>, Vec<_>, Vec<_>) =
                multiunzip(conditions_and_carry_overs?);

            let (condition, vars, tuples) = match parent_node {
                ReqDeclNode::And(_) => {
                    let conditions = conditions.into_iter().flatten().collect_vec();
                    let condition = match conditions[..] {
                        [] => None,
                        _ => Some(build_condition_expression(&conditions[..], &|lhs, rhs| {
                            instantiate.lazy_and(lhs, rhs)
                        })),
                    };
                    let mut vars = carry_over_vars.into_iter().flatten().collect_vec();
                    let mut tuples = carry_over_tuples.into_iter().flatten().collect_vec();

                    if parent_node_is_root_node {
                        // We are within an AND root node. Add all the variable declarations to the result var declarations and
                        // return the calculated condition and no carry overs.
                        // `vars` and `tuples` will be empty after appending.

                        // Note that if we have more than one tuple in carry over, this means they
                        // are coming from an AND node (because an OR node always produces a single tuple).
                        // In that case the `vars` redefined in tuples are never the same and we can
                        // safely declare them in any order after the tuples.
                        result_var_declarations.append(&mut tuples);
                        result_var_declarations.append(&mut vars);
                    }

                    // Return the condition and either the empty `vars` and `tuples` if the parent is the root node, or carry over
                    // all the declarations from all the child nodes.
                    (condition, vars, tuples)
                }
                ReqDeclNode::Or(_) => {
                    let has_var_decls = carry_over_vars.iter().any(|v| !v.is_empty());

                    if has_var_decls {
                        // We need to:
                        // - instantiate the index variable for this OR.
                        // - instantiate a single tuple variable that holds the variables taken from the alternatives.
                        // - instantiate redefined declared variables that are initialized from the tuple fields.

                        // Instantiate and return the expression for matched OR variant index variable.
                        let suffix = or_variant_index_vars.len() + 1;
                        let matched_or_variant_index_var_decl =
                            instantiate_matched_or_variant_index_var_expression(
                                instantiate,
                                suffix,
                                conditions,
                            );
                        // Variable expression used to instantiate the corresponding tuple variable
                        // that will hold matched variant variables.
                        // Note that it is not needed to add the declaration of this variable
                        // to the context in order for the tuple variable to be created.
                        let matched_or_variant_index_variable = instantiate.var_exp(
                            matched_or_variant_index_var_decl.0.clone(),
                            matched_or_variant_index_var_decl.1.return_type,
                        );

                        or_variant_index_vars.push(matched_or_variant_index_var_decl);

                        // Instantiate the tuple variable and the redefined variable declarations
                        // of the variables declared in OR variants.

                        let (tuple, mut redefined_vars) =
                            instantiate_matched_or_variant_vars_expressions(
                                handler,
                                ctx.by_ref(),
                                instantiate,
                                &matched_or_variant_index_variable,
                                suffix,
                                carry_over_vars,
                                carry_over_tuples,
                            )?;

                        // Instantiate the new condition that will be just the check if the 1-based matched variant index is different
                        // then zero.
                        let condition = instantiate.neq_result(
                            handler,
                            ctx.by_ref(),
                            matched_or_variant_index_variable,
                            instantiate.u64_literal(0),
                        )?;

                        if parent_node_is_root_node {
                            // We are within an OR root node. Add the tuple and all the variable declarations to the result var declarations and
                            // return the calculated condition and no carry overs.
                            result_var_declarations.push(tuple);
                            result_var_declarations.append(&mut redefined_vars);

                            (Some(condition), vec![], vec![])
                        } else {
                            // Return the condition and or carry over the created tuple and
                            // all the redefined variable declarations to the upper nodes.
                            (Some(condition), redefined_vars, vec![tuple])
                        }
                    } else {
                        // No variable declarations in OR variants.
                        // This also means we don't have tuples because they are created only to extract variables.
                        // In this case we only have to calculate the final condition.
                        let conditions = conditions.into_iter().flatten().collect_vec();
                        let condition = match conditions[..] {
                            [] => None,
                            _ => Some(build_condition_expression(&conditions[..], &|lhs, rhs| {
                                instantiate.lazy_or(lhs, rhs)
                            })),
                        };

                        (condition, vec![], vec![])
                    }
                }
                _ => unreachable!("A parent node can only be an AND or an OR node."),
            };

            Ok((condition, vars, tuples))
        }

        fn build_condition_expression(
            expressions: &[ty::TyExpression],
            operator: &impl Fn(ty::TyExpression, ty::TyExpression) -> ty::TyExpression,
        ) -> ty::TyExpression {
            let (lhs, others) = expressions
                .split_first()
                .expect("The slice of requirement expressions must not be empty.");
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
        /// ```ignore
        /// let __matched_or_variant_index_<suffix>: u64 = if <variant_1_condition> {
        ///         1u64
        ///     } else if <variant_2_condition> {
        ///         2u64
        ///     } else if ... {
        ///         ...
        ///     } else {
        ///         0u64
        ///     };
        /// ```
        fn instantiate_matched_or_variant_index_var_expression(
            instantiate: &Instantiate,
            suffix: usize,
            conditions: Vec<MatchBranchCondition>,
        ) -> (Ident, ty::TyExpression) {
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
                        then: Box::new(instantiate.code_block_with_implicit_return_u64(
                            (number_of_alternatives - rev_index).try_into().unwrap(),
                        )),
                        r#else: Some(Box::new(if_expr)), // Put the previous if into else.
                    },
                    return_type: instantiate.u64_type(),
                    span: instantiate.dummy_span(),
                }
            }

            (ident, if_expr)
        }

        /// Instantiates immutable variable declarations for all the variables
        /// declared in an OR match expression.
        /// Choosing the right initialization, the initialization coming from
        /// the OR variant that actually matched, is done by inspecting
        /// the result of the corresponding __matched_or_variant_index_<suffix>
        /// variable.
        ///
        /// The function returns:
        /// - a variable declaration of the tuple variable that holds
        ///   the values of all the variables declared in the OR match expression
        /// - redefined declarations of each individual variable.
        ///
        /// ```ignore
        /// let __matched_or_variant_variables_<suffix>: <tuple> = if __matched_or_variant_index_<suffix> == 1 {
        ///         <potential tuple declarations carried over from the child nodes>
        ///
        ///         (<var_1_variant_1_initialization>, ..., <var_n_variant_1_initialization>)
        ///     } else if __match_matched_or_variant_index_<suffix> == 2 {
        ///         <potential tuple declarations carried over from the child nodes>
        ///
        ///         (<var_1_variant_2_initialization>, ..., <var_n_variant_2_initialization>)
        ///     } else if ... {
        ///         ...
        ///     } else {
        ///         __revert(...) // This should never happen and means internal compiler error.
        ///     };
        ///
        /// let <var_1> = __matched_or_variant_variables_<suffix>.0;
        /// let <var_2> = __matched_or_variant_variables_<suffix>.1;
        /// ...
        /// let <var_n> = __matched_or_variant_variables_<suffix>.(n-1);
        /// ```
        fn instantiate_matched_or_variant_vars_expressions(
            handler: &Handler,
            mut ctx: TypeCheckContext,
            instantiate: &Instantiate,
            matched_or_variant_index_var: &ty::TyExpression,
            suffix: usize,
            mut carry_over_vars: Vec<CarryOverVarDeclarations>,
            carry_over_tuples: Vec<CarryOverTupleDeclarations>,
        ) -> Result<(VarDecl, Vec<VarDecl>), ErrorEmitted> {
            let type_engine = ctx.engines.te();
            // At this point we have the guarantee that we have:
            // - exactly the same variables in each of the OR variants
            // - that variables of the same name are of the same type
            // - that we do not have duplicates in variable names inside of alternatives

            // Sort variables in all alternatives by name to get deterministic ordering in the resulting tuple.
            // Note that the var declarations in match patterns are mutually independent, thus,
            // we can shuffle their ordering.

            for vars_in_alternative in carry_over_vars.iter_mut() {
                vars_in_alternative.sort_by(|(a, _), (b, _)| a.cmp(b));
            }

            // Still, check the above guarantee and emit internal compiler errors if they are not satisfied.
            check_variables_guarantee(
                handler,
                ctx.engines,
                &carry_over_vars,
                instantiate.error_span(),
            )?;

            // Build the `if-else` chain for the declaration of the tuple variable.
            // Build it bottom up, means traverse in reverse order.

            // All variants have same variable types and names, thus we pick them from the first alternative.
            let tuple_field_types = carry_over_vars[0]
                .iter()
                .map(|(_, var_body)| var_body.return_type)
                .collect();
            let tuple_type =
                type_engine.insert_tuple_without_annotations(ctx.engines, tuple_field_types);
            let variable_names = carry_over_vars[0]
                .iter()
                .map(|(ident, _)| ident.clone())
                .collect_vec();

            // Build the expression bottom up by putting the previous if expression into
            // the else part of the current one.
            let number_of_alternatives = carry_over_vars.len();
            let mut if_expr = instantiate
                .code_block_with_implicit_return_revert(INVALID_MATCHED_OR_VARIABLE_INDEX_SIGNAL);
            // The vectors of vars and tuples defined in alternatives, are of the same size, which is the number of OR alternatives.
            for (rev_index, (vars_in_alternative, tuples_in_alternative)) in carry_over_vars
                .into_iter()
                .rev()
                .zip_eq(carry_over_tuples.into_iter().rev())
                .enumerate()
            {
                let condition = instantiate_or_variant_has_matched_condition(
                    ctx.by_ref(),
                    instantiate,
                    matched_or_variant_index_var,
                    (number_of_alternatives - rev_index).try_into().unwrap(),
                );

                let mut code_block_contents = vec![];

                // Add carry over tuples, if any.
                for tuple in tuples_in_alternative {
                    code_block_contents.push(ty::TyAstNode {
                        content: ty::TyAstNodeContent::Declaration(
                            instantiate.var_decl(tuple.0, tuple.1),
                        ),
                        span: instantiate.dummy_span(),
                    });
                }

                // Add the implicit return tuple that captures the values of the variables.
                let ret_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::Tuple {
                        fields: vars_in_alternative
                            .into_iter()
                            .map(|(_, exp)| exp)
                            .collect(),
                    },
                    return_type: tuple_type,
                    span: instantiate.dummy_span(),
                };
                code_block_contents.push(ty::TyAstNode {
                    content: ty::TyAstNodeContent::Expression(ty::TyExpression {
                        return_type: ret_expr.return_type,
                        span: ret_expr.span.clone(),
                        expression: ty::TyExpressionVariant::ImplicitReturn(Box::new(ret_expr)),
                    }),
                    span: instantiate.dummy_span(),
                });

                if_expr = ty::TyExpression {
                    expression: ty::TyExpressionVariant::IfExp {
                        condition: Box::new(condition),
                        then: Box::new(ty::TyExpression {
                            expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                                whole_block_span: instantiate.dummy_span(),
                                contents: code_block_contents,
                            }),
                            return_type: tuple_type,
                            span: instantiate.dummy_span(),
                        }),
                        r#else: Some(Box::new(if_expr)), // Put the previous if into else.
                    },
                    return_type: tuple_type,
                    span: instantiate.dummy_span(),
                }
            }

            let matched_or_variant_variables_tuple_ident =
                instantiate.ident(generate_matched_or_variant_variables_var_name(suffix));

            // For every variable in alternatives, redefined it by initializing it to the corresponding tuple field.
            let mut redefined_variables = vec![];

            // Variable expression used to emit tuple index access.
            // Note that it is not needed to add the tuple declaration to the
            // context in order for the index access expression to be created.
            let tuple_variable =
                instantiate.var_exp(matched_or_variant_variables_tuple_ident.clone(), tuple_type);

            for (index, variable) in variable_names.into_iter().enumerate() {
                redefined_variables.push((
                    variable,
                    instantiate.tuple_elem_access(ctx.engines, tuple_variable.clone(), index),
                ));
            }

            return Ok((
                (matched_or_variant_variables_tuple_ident, if_expr),
                redefined_variables,
            ));

            /// Creates a boolean condition of the form `<matched_or_variant_index_variable> == <variant_index>`.
            /// `matched_or_variant_index_variable` is the corresponding variable of the name `__match_matched_or_variant_index_<suffix>`.
            fn instantiate_or_variant_has_matched_condition(
                ctx: TypeCheckContext,
                instantiate: &Instantiate,
                matched_or_variant_index_variable: &ty::TyExpression,
                variant_index: u64,
            ) -> ty::TyExpression {
                let variant_index_exp = instantiate.u64_literal(variant_index);
                instantiate.eq(
                    ctx,
                    matched_or_variant_index_variable.clone(),
                    variant_index_exp,
                )
            }

            fn check_variables_guarantee(
                handler: &Handler,
                engines: &Engines,
                sorted_var_declarations: &[CarryOverVarDeclarations],
                error_span: Span,
            ) -> Result<(), ErrorEmitted> {
                // Guarantees:
                // - exactly the same variables in each OR variant
                // - variables of the same name are of the same type
                // - we do not have duplicates in variable names inside of alternatives
                let (first_alternative_vars, other_alternatives_vars) = sorted_var_declarations
                    .split_first()
                    .expect("Variable declarations must come from at least two OR alternatives.");

                if other_alternatives_vars
                    .iter()
                    .any(|vars| vars.len() != first_alternative_vars.len())
                {
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
                for index in 0..first_alternative_vars.len() - 1 {
                    if first_alternative_vars[index].0 == first_alternative_vars[index + 1].0 {
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
