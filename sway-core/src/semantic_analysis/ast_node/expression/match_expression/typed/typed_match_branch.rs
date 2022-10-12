use sway_types::Spanned;

use crate::{
    error::{err, ok},
    language::{parsed::MatchBranch, ty},
    semantic_analysis::{
        ast_node::expression::match_expression::typed::typed_scrutinee::TyScrutinee, IsConstant,
        TyAstNode, TyAstNodeContent, TyCodeBlock, TyVariableDeclaration, TypeCheckContext,
        VariableMutability,
    },
    type_system::insert_type,
    types::DeterministicallyAborts,
    CompileResult, TypeInfo,
};

use super::matcher::matcher;

impl ty::TyMatchBranch {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        typed_value: &ty::TyExpression,
        branch: MatchBranch,
    ) -> CompileResult<(ty::TyMatchBranch, TyScrutinee)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let MatchBranch {
            scrutinee,
            result,
            span: branch_span,
        } = branch;

        // type check the scrutinee
        let typed_scrutinee = check!(
            TyScrutinee::type_check(ctx.by_ref(), scrutinee),
            return err(warnings, errors),
            warnings,
            errors
        );

        // calculate the requirements map and the declarations map
        let (match_req_map, match_decl_map) = check!(
            matcher(typed_value, typed_scrutinee.clone(), ctx.namespace),
            return err(warnings, errors),
            warnings,
            errors
        );

        // create a new namespace for this branch
        let mut namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut namespace);

        // for every item in the declarations map, create a variable declaration,
        // insert it into the branch namespace, and add it to a block of code statements
        let mut code_block_contents: Vec<TyAstNode> = vec![];
        for (left_decl, right_decl) in match_decl_map.into_iter() {
            let type_ascription = right_decl.return_type;
            let span = left_decl.span().clone();
            let var_decl =
                ty::TyDeclaration::VariableDeclaration(Box::new(TyVariableDeclaration {
                    name: left_decl.clone(),
                    body: right_decl,
                    mutability: VariableMutability::Immutable,
                    type_ascription,
                    type_ascription_span: None,
                }));
            ctx.namespace.insert_symbol(left_decl, var_decl.clone());
            code_block_contents.push(TyAstNode {
                content: TyAstNodeContent::Declaration(var_decl),
                span,
            });
        }

        // type check the branch result
        let typed_result = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                ty::TyExpression::type_check(ctx, result),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        // unify the return type from the typed result with the type annotation
        if !typed_result.deterministically_aborts() {
            append!(
                ctx.unify_with_self(typed_result.return_type, &typed_result.span),
                warnings,
                errors
            );
        }

        // if the typed branch result is a code block, then add the contents
        // of that code block to the block of code statements that we are already
        // generating. if the typed branch result is not a code block, then add
        // the typed branch result as an ast node to the block of code statements
        let ty::TyExpression {
            expression: typed_result_expression_variant,
            return_type: typed_result_return_type,
            is_constant: typed_result_is_constant,
            span: typed_result_span,
        } = typed_result;
        match typed_result_expression_variant {
            ty::TyExpressionVariant::CodeBlock(TyCodeBlock { mut contents, .. }) => {
                code_block_contents.append(&mut contents);
            }
            typed_result_expression_variant => {
                code_block_contents.push(TyAstNode {
                    content: TyAstNodeContent::ImplicitReturnExpression(ty::TyExpression {
                        expression: typed_result_expression_variant,
                        return_type: typed_result_return_type,
                        is_constant: typed_result_is_constant,
                        span: typed_result_span.clone(),
                    }),
                    span: typed_result_span.clone(),
                });
            }
        }

        // assemble a new branch result that includes both the variable declarations
        // that we create and the typed result from the original untyped branch
        let new_result = ty::TyExpression {
            expression: ty::TyExpressionVariant::CodeBlock(TyCodeBlock {
                contents: code_block_contents,
            }),
            return_type: typed_result.return_type,
            is_constant: IsConstant::No,
            span: typed_result_span,
        };

        // return!
        let typed_branch = ty::TyMatchBranch {
            conditions: match_req_map,
            result: new_result,
            span: branch_span,
        };
        ok((typed_branch, typed_scrutinee), warnings, errors)
    }
}
