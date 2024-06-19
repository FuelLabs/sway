use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

use itertools::Itertools;
use sway_ast::{literal::LitString, Literal};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{BaseIdent, Ident, Span, Spanned};

use crate::{
    compiler_generated::INVALID_DESUGARED_MATCHED_EXPRESSION_SIGNAL,
    language::{
        parsed::*,
        ty::{
            self, TyAsmRegisterDeclaration, TyAstNode, TyCodeBlock, TyExpression,
            TyExpressionVariant, TyIntrinsicFunctionKind, TyMatchExpression,
        },
        AsmOp, AsmRegister,
    },
    semantic_analysis::{
        ast_node::expression::typed_expression::instantiate_if_expression,
        expression::match_expression::typed::instantiate::Instantiate, TypeCheckContext,
    },
    CompileError, TypeArgument, TypeId, TypeInfo,
};

#[derive(Default, Debug, Clone)]
struct TrieNode {
    output: Option<usize>,
    previous: Option<usize>,
    next: BTreeMap<String, usize>,
}

impl ty::TyMatchExpression {
    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        typed_value: ty::TyExpression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> Result<(ty::TyMatchExpression, Vec<ty::TyScrutinee>), ErrorEmitted> {
        // type check all of the branches
        let mut typed_branches = vec![];
        let mut typed_scrutinees = vec![];
        let mut ctx =
            ctx.with_help_text("all branches of a match statement must return the same type");

        handler.scope(|handler| {
            for branch in branches.into_iter() {
                let (typed_branch, typed_scrutinee) = match ty::TyMatchBranch::type_check(
                    handler,
                    ctx.by_ref(),
                    &typed_value,
                    branch,
                ) {
                    Ok(res) => res,
                    Err(_) => continue,
                };
                typed_branches.push(typed_branch);
                typed_scrutinees.push(typed_scrutinee);
            }

            Ok(())
        })?;

        let typed_exp = ty::TyMatchExpression {
            value_type_id: typed_value.return_type,
            branches: typed_branches,
            return_type_id: ctx.type_annotation(),
            span,
        };

        Ok((typed_exp, typed_scrutinees))
    }

    pub(crate) fn desugar(
        self,
        handler: &Handler,
        ctx: TypeCheckContext,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let instantiate = Instantiate::new(ctx.engines, self.span.clone());

        if self.branches.is_empty() {
            return Self::instantiate_if_expression_for_empty_match_expression(
                handler,
                ctx,
                &instantiate,
                self.value_type_id,
                self.return_type_id,
                self.span.clone(),
            );
        }

        let typed_if_exp =
            handler.scope(
                |handler| match &*ctx.engines().te().get(self.value_type_id) {
                    TypeInfo::StringSlice => self.desugar_to_radix_tree(instantiate, ctx, handler),
                    _ => self.desugar_to_typed_if_expression(instantiate, ctx, handler),
                },
            )?;

        Ok(typed_if_exp)
    }

    fn desugar_to_radix_tree(
        &self,
        instantiate: Instantiate,
        mut ctx: TypeCheckContext<'_>,
        handler: &Handler,
    ) -> Result<TyExpression, ErrorEmitted> {
        fn revert(never_type_id: TypeId) -> TyAstNode {
            TyAstNode {
                content: ty::TyAstNodeContent::Expression(TyExpression {
                    expression: TyExpressionVariant::IntrinsicFunction(TyIntrinsicFunctionKind {
                        kind: sway_ast::Intrinsic::Revert,
                        arguments: vec![TyExpression {
                            expression: TyExpressionVariant::Literal(
                                crate::language::Literal::U64(17),
                            ),
                            return_type: never_type_id,
                            span: Span::dummy(),
                        }],
                        type_arguments: vec![],
                        span: Span::dummy(),
                    }),
                    return_type: never_type_id,
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }
        }

        let never_type_id = ctx.engines.te().insert(&ctx.engines, TypeInfo::Never, None);

        let branch_return_type_id = self
            .branches
            .iter()
            .map(|x| x.result.return_type)
            .next()
            .unwrap();

        let matched_value = self
            .branches
            .iter()
            .flat_map(|x| match &x.condition.as_ref().map(|x| &x.expression) {
                Some(TyExpressionVariant::FunctionApplication { arguments, .. }) => {
                    Some(&arguments[0].1)
                }
                _ => None,
            })
            .next()
            .unwrap();

        let wildcard_ast_node = self
            .branches
            .iter()
            .filter(|x| x.condition.is_none())
            .map(|x| TyAstNode {
                content: ty::TyAstNodeContent::Expression(x.result.clone()),
                span: Span::dummy(),
            })
            .next()
            .unwrap_or_else(|| revert(never_type_id));

        let branches = self
            .branches
            .iter()
            .flat_map(|x| match &x.condition.as_ref().map(|x| &x.expression) {
                Some(TyExpressionVariant::FunctionApplication { arguments, .. }) => {
                    match &arguments[1].1.expression {
                        TyExpressionVariant::Literal(crate::language::Literal::String(v)) => {
                            Some(v.as_str().to_string())
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut nodes = vec![TrieNode::default()];
        for (i, b) in branches.iter().enumerate() {
            let mut current = 0;
            for c in b.chars() {
                let c = c.to_string();
                if let Some(next) = nodes[current].next.get(&c) {
                    current = *next;
                    continue;
                }

                let next = nodes.len();
                nodes[current].next.insert(c, next);
                current = next;
                nodes.push(TrieNode::default());
            }

            nodes[current].output = Some(i);
        }

        let mut packed_strings = BTreeSet::new();

        // compress trie
        let mut q = vec![0];
        while let Some(i) = q.pop() {
            let mut current = nodes[i].clone();
            if current.next.len() == 1 {
                let edge = current.next.pop_first().unwrap();
                let mut next = nodes[edge.1].clone();
                if next.next.len() == 1 {
                    let next_edge = next.next.pop_first().unwrap();
                    let compressed_key = format!("{}{}", edge.0, next_edge.0);

                    nodes[i].next.clear();
                    nodes[i].next.insert(compressed_key, next_edge.1);
                    nodes[i].output = next.output.take();

                    q.push(i);
                } else {
                    packed_strings.insert(edge.0.clone());

                    nodes[edge.1].previous = Some(i);
                    q.push(edge.1);
                }
            } else {
                for (prefix, v) in current.next.iter() {
                    packed_strings.insert(prefix.clone());

                    nodes[*v].previous = Some(i);
                    q.push(*v);
                }
            }
        }

        //generate code
        fn generate_code(
            s: &TyMatchExpression,
            matched_value: &TyExpression,
            packed_strings: &str,
            addr_of_packed_strings: &TyExpression,
            nodes: &[TrieNode],
            slice_pos: usize,
            current_node_index: usize,
            never_type_id: TypeId,
            bool_type_id: TypeId,
            u64_type_id: TypeId,
            branch_return_type_id: TypeId,
            depth: usize,
            block_when_all_fail: TyAstNode,
        ) -> TyAstNode {
            let current = &nodes[current_node_index];
            let mut contents = vec![];

            if let Some(output) = current.output {
                assert!(current.next.len() == 0);
                // println!("{}return {:?}", " ".repeat(depth * 4), output);
                return TyAstNode {
                    content: ty::TyAstNodeContent::Expression(s.branches[output].result.clone()),
                    span: Span::dummy(),
                };
            }

            for (prefix, next_node_index) in current.next.iter() {
                let start = current_node_index;
                let end = current_node_index + prefix.len();
                let eq_len: u64 = end as u64 - start as u64;
                // println!(
                //     "{}if str[{start}..{end}] == \"{}\" {{ }}",
                //     " ".repeat(depth * 4),
                //     prefix,
                // );

                let inner_node = generate_code(
                    s,
                    matched_value,
                    packed_strings,
                    addr_of_packed_strings,
                    nodes,
                    end,
                    *next_node_index,
                    never_type_id,
                    bool_type_id,
                    u64_type_id,
                    branch_return_type_id,
                    depth + 1,
                    revert(never_type_id),
                );

                let prefix_pos = packed_strings
                    .find(prefix)
                    .expect("prefix should be inside this string");

                let expression = TyExpressionVariant::AsmExpression {
                    registers: vec![
                        TyAsmRegisterDeclaration {
                            name: Ident::new_no_span("slice".into()),
                            initializer: Some(matched_value.clone()),
                        },
                        TyAsmRegisterDeclaration {
                            name: Ident::new_no_span("prefix".into()),
                            initializer: Some(addr_of_packed_strings.clone()),
                        },
                        TyAsmRegisterDeclaration {
                            name: Ident::new_no_span("slice_ptr".into()),
                            initializer: None,
                        },
                        TyAsmRegisterDeclaration {
                            name: Ident::new_no_span("prefix_ptr".into()),
                            initializer: None,
                        },
                        TyAsmRegisterDeclaration {
                            name: Ident::new_no_span("len".into()),
                            initializer: Some(TyExpression {
                                expression: TyExpressionVariant::Literal(
                                    crate::language::Literal::U64(eq_len),
                                ),
                                return_type: u64_type_id,
                                span: Span::dummy(),
                            }),
                        },
                        TyAsmRegisterDeclaration {
                            name: Ident::new_no_span("is_eq".into()),
                            initializer: None,
                        },
                    ],
                    body: vec![
                        AsmOp {
                            op_name: Ident::new_no_span("addi".into()),
                            op_args: vec![
                                BaseIdent::new_no_span("slice_ptr".into()),
                                BaseIdent::new_no_span("slice".into()),
                            ],
                            immediate: Some(BaseIdent::new_no_span(format!("i{}", slice_pos))),
                            span: Span::dummy(),
                        },
                        AsmOp {
                            op_name: Ident::new_no_span("addi".into()),
                            op_args: vec![
                                BaseIdent::new_no_span("prefix_ptr".into()),
                                BaseIdent::new_no_span("prefix".into()),
                            ],
                            immediate: Some(BaseIdent::new_no_span(format!("i{}", prefix_pos))),
                            span: Span::dummy(),
                        },
                        AsmOp {
                            op_name: Ident::new_no_span("meq".into()),
                            op_args: vec![
                                BaseIdent::new_no_span("is_eq".into()),
                                BaseIdent::new_no_span("slice_ptr".into()),
                                BaseIdent::new_no_span("prefix_ptr".into()),
                                BaseIdent::new_no_span("len".into()),
                            ],
                            immediate: None,
                            span: Span::dummy(),
                        },
                    ],
                    returns: Some((
                        AsmRegister {
                            name: "is_eq".into(),
                        },
                        Span::dummy(),
                    )),
                    whole_block_span: Span::dummy(),
                };

                let expr = TyExpression {
                    expression: TyExpressionVariant::IfExp {
                        condition: Box::new(TyExpression {
                            expression,
                            return_type: bool_type_id,
                            span: Span::dummy(),
                        }),
                        then: Box::new(TyExpression {
                            expression: TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                                contents: vec![inner_node],
                                whole_block_span: Span::dummy(),
                            }),
                            return_type: branch_return_type_id,
                            span: Span::dummy(),
                        }),
                        r#else: None,
                    },
                    return_type: branch_return_type_id,
                    span: Span::dummy(),
                };

                contents.push(TyAstNode {
                    content: ty::TyAstNodeContent::Expression(expr),
                    span: Span::dummy(),
                });
            }

            if current.output.is_none() {
                contents.push(block_when_all_fail);
            }

            let block = TyExpression {
                expression: TyExpressionVariant::CodeBlock(TyCodeBlock {
                    contents,
                    whole_block_span: Span::dummy(),
                }),
                return_type: branch_return_type_id,
                span: Span::dummy(),
            };
            TyAstNode {
                content: ty::TyAstNodeContent::Expression(block),
                span: Span::dummy(),
            }
        }

        let bool_type_id = ctx
            .engines
            .te()
            .insert(&ctx.engines, TypeInfo::Boolean, None);

        let u64_type_id = ctx.engines.te().insert(
            &ctx.engines,
            TypeInfo::UnsignedInteger(sway_types::integer_bits::IntegerBits::SixtyFour),
            None,
        );

        let string_slice_type_id =
            ctx.engines
                .te()
                .insert(&ctx.engines, TypeInfo::StringSlice, None);

        let ptr_string_slice_type_id = TypeInfo::Ptr(TypeArgument {
            type_id: string_slice_type_id,
            initial_type_id: string_slice_type_id,
            span: Span::dummy(),
            call_path_tree: None,
        });
        let ptr_string_slice_type_id =
            ctx.engines
                .te()
                .insert(&ctx.engines, ptr_string_slice_type_id, None);

        let packed_strings: String = packed_strings.iter().join("");
        let packed_strings_expr = TyExpression {
            expression: TyExpressionVariant::Literal(crate::language::Literal::String(
                Span::from_string(packed_strings.clone()),
            )),
            return_type: string_slice_type_id,
            span: Span::dummy(),
        };
        let addr_of_packed_strings = TyExpression {
            expression: TyExpressionVariant::IntrinsicFunction(TyIntrinsicFunctionKind {
                kind: sway_ast::Intrinsic::AddrOf,
                arguments: vec![packed_strings_expr],
                type_arguments: vec![],
                span: Span::dummy(),
            }),
            return_type: ptr_string_slice_type_id,
            span: Span::dummy(),
        };

        let expr = generate_code(
            self,
            matched_value,
            &packed_strings,
            &addr_of_packed_strings,
            &nodes,
            0,
            0,
            never_type_id,
            bool_type_id,
            u64_type_id,
            branch_return_type_id,
            0,
            dbg!(wildcard_ast_node),
        );

        let block = TyCodeBlock {
            contents: vec![expr],
            whole_block_span: Span::dummy(),
        };
        Ok(TyExpression {
            expression: TyExpressionVariant::CodeBlock(block),
            return_type: self.return_type_id,
            span: Span::dummy(),
        })
    }

    fn desugar_to_typed_if_expression(
        &self,
        instantiate: Instantiate,
        mut ctx: TypeCheckContext<'_>,
        handler: &Handler,
    ) -> Result<TyExpression, ErrorEmitted> {
        // The typed if expression object that we will be building on to.
        // We will do it bottom up, starting from the final `else`.
        let mut typed_if_exp = None;

        // For every branch, bottom-up, means in reverse.
        for ty::TyMatchBranch {
            matched_or_variant_index_vars,
            condition,
            result,
            ..
        } in self.branches.iter().rev()
        {
            if let ControlFlow::Break(_) = self.convert_to_typed_if_expression_inner_branch(
                &mut typed_if_exp,
                condition,
                result,
                &instantiate,
                &mut ctx,
                handler,
                matched_or_variant_index_vars,
            )? {
                continue;
            }
        }

        Ok(typed_if_exp.expect("The expression exists because we have at least one branch."))
    }

    #[allow(clippy::too_many_arguments)]
    fn convert_to_typed_if_expression_inner_branch(
        &self,
        typed_if_exp: &mut Option<TyExpression>,
        condition: &Option<TyExpression>,
        result: &TyExpression,
        instantiate: &Instantiate,
        ctx: &mut TypeCheckContext<'_>,
        handler: &Handler,
        matched_or_variant_index_vars: &Vec<(sway_types::BaseIdent, TyExpression)>,
    ) -> Result<ControlFlow<()>, ErrorEmitted> {
        if typed_if_exp.is_none() {
            // If the last match arm is a catch-all arm make its result the final else.
            // Note that this will always be the case with `if let` expressions that
            // desugar to match expressions.
            if condition.is_none() {
                *typed_if_exp = Some(result.clone());
                return Ok(ControlFlow::Break(())); // Last branch added, move to the previous one.
            } else {
                // Otherwise instantiate the final `__revert`.
                let final_revert = instantiate.code_block_with_implicit_return_revert(
                    INVALID_DESUGARED_MATCHED_EXPRESSION_SIGNAL,
                );

                *typed_if_exp = Some(final_revert);
                // Continue with adding the last branch.
            };
        }
        let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
        ctx.scoped(|mut branch_ctx| {
            let result_span = result.span.clone();
            let condition = condition
                .clone()
                .unwrap_or(instantiate.boolean_literal(true));
            let if_exp = match instantiate_if_expression(
                handler,
                branch_ctx.by_ref(),
                condition,
                result.clone(),
                Some(
                    typed_if_exp
                        .clone()
                        .expect("The previously created expression exist at this point."),
                ), // Put the previous if into else.
                result_span.clone(),
            ) {
                Ok(if_exp) => if_exp,
                Err(_) => {
                    return Ok(ControlFlow::Break(()));
                }
            };
            // If we are instantiating the final `else` block.

            // Create a new namespace for this branch result.

            *typed_if_exp = if matched_or_variant_index_vars.is_empty() {
                // No OR variants with vars. We just have to instantiate the if expression.
                Some(if_exp)
            } else {
                // We have matched OR variant index vars.
                // We need to add them to the block before the if expression.
                // The resulting `typed_if_exp` in this case is actually not
                // an if expression but rather a code block.
                let mut code_block_contents: Vec<ty::TyAstNode> = vec![];

                for (var_ident, var_body) in matched_or_variant_index_vars {
                    let var_decl = instantiate.var_decl(var_ident.clone(), var_body.clone());
                    let span = var_ident.span();
                    let _ = branch_ctx.insert_symbol(handler, var_ident.clone(), var_decl.clone());
                    code_block_contents.push(ty::TyAstNode {
                        content: ty::TyAstNodeContent::Declaration(var_decl),
                        span,
                    });
                }

                code_block_contents.push(ty::TyAstNode {
                    content: ty::TyAstNodeContent::Expression(TyExpression {
                        return_type: if_exp.return_type,
                        span: if_exp.span.clone(),
                        expression: ty::TyExpressionVariant::ImplicitReturn(Box::new(if_exp)),
                    }),
                    span: result_span.clone(),
                });

                Some(ty::TyExpression {
                    expression: ty::TyExpressionVariant::CodeBlock(ty::TyCodeBlock {
                        whole_block_span: Span::dummy(),
                        contents: code_block_contents,
                    }),
                    return_type: self.return_type_id,
                    span: result_span.clone(),
                })
            };
            Ok(ControlFlow::Continue(()))
        })
    }

    fn instantiate_if_expression_for_empty_match_expression(
        handler: &Handler,
        ctx: TypeCheckContext,
        instantiate: &Instantiate,
        value_type_id: TypeId,
        return_type_id: TypeId,
        span: Span,
    ) -> Result<ty::TyExpression, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();

        // An empty match expression can happen only if the type we
        // are matching on does not have a valid constructor.
        // Otherwise, the match expression must be exhaustive, means
        // it must have at least one match arm.
        // In this case, we manually create a typed expression that is equivalent to
        // `if true { implicit_return }` where the implicit_return type is manually set
        // to be the return type of this typed match expression object.
        //
        // An example of such matching is when matching an empty enum.
        // For an example, see the "match_expressions_empty_enums" test.
        //
        // NOTE: This manual construction of the expression can (and
        // most likely will) lead to an otherwise improperly typed
        // expression, in most cases.
        if !type_engine
            .get(value_type_id)
            .has_valid_constructor(decl_engine)
        {
            let condition = instantiate.boolean_literal(true);
            let then_exp = ty::TyExpression {
                expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                return_type: return_type_id,
                span: instantiate.dummy_span(),
            };
            let inner_exp = ty::TyExpressionVariant::IfExp {
                condition: Box::new(condition),
                then: Box::new(then_exp.clone()),
                r#else: Option::Some(Box::new(then_exp)),
            };
            let typed_if_exp = ty::TyExpression {
                expression: inner_exp,
                return_type: return_type_id,
                span: instantiate.dummy_span(),
            };

            return Ok(typed_if_exp);
        }

        Err(handler.emit_err(CompileError::Internal(
            "unable to convert match exp to if exp",
            span,
        )))
    }
}
