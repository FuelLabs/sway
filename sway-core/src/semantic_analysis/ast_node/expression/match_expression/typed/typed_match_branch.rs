use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Spanned;

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

        // calculate the requirements map and the declarations map
        let (match_req_map, match_decl_map) =
            matcher(handler, ctx.by_ref(), typed_value, typed_scrutinee.clone())?;

        // create a new namespace for this branch
        let mut namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut namespace);

        // for every item in the declarations map, create a variable declaration,
        // insert it into the branch namespace, and add it to a block of code statements
        let mut code_block_contents: Vec<ty::TyAstNode> = vec![];
        for (left_decl, right_decl) in match_decl_map.into_iter() {
            let type_ascription = right_decl.return_type.into();
            let return_type = right_decl.return_type;
            let span = left_decl.span().clone();
            let var_decl = ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
                name: left_decl.clone(),
                body: right_decl,
                mutability: ty::VariableMutability::Immutable,
                return_type,
                type_ascription,
            }));
            let _ = ctx.insert_symbol(handler, left_decl, var_decl.clone());
            code_block_contents.push(ty::TyAstNode {
                content: ty::TyAstNodeContent::Declaration(var_decl),
                span,
            });
        }

        // type check the branch result
        let typed_result = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown));
            ty::TyExpression::type_check(handler, ctx, result)?
        };

        // unify the return type from the typed result with the type annotation
        if !typed_result.deterministically_aborts(decl_engine, true) {
            ctx.unify_with_self(handler, typed_result.return_type, &typed_result.span);
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

        // return!
        let typed_branch = ty::TyMatchBranch {
            cnf: match_req_map,
            result: new_result,
            span: branch_span,
        };
        Ok((typed_branch, typed_scrutinee))
    }
}
