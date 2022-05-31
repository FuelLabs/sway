use sway_types::Span;

use crate::{
    error::{err, ok},
    semantic_analysis::{
        ast_node::expression::match_expression::typed::typed_scrutinee::TypedScrutinee, IsConstant,
        TypeCheckArguments, TypedAstNode, TypedAstNodeContent, TypedCodeBlock, TypedExpression,
        TypedExpressionVariant, TypedVariableDeclaration, VariableMutability,
    },
    type_engine::{insert_type, unify_with_self},
    CompileResult, MatchBranch, TypeInfo, TypedDeclaration,
};

use super::matcher::{matcher, MatchReqMap};

#[derive(Debug)]
pub(crate) struct TypedMatchBranch {
    pub(crate) conditions: MatchReqMap,
    pub(crate) result: TypedExpression,
    #[allow(dead_code)]
    span: Span,
}

impl TypedMatchBranch {
    pub(crate) fn type_check(
        arguments: TypeCheckArguments<'_, (&TypedExpression, MatchBranch)>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TypeCheckArguments {
            checkee: (typed_value, branch),
            namespace,
            return_type_annotation,
            self_type,
            opts,
            help_text,
            mode,
        } = arguments;

        let MatchBranch {
            scrutinee,
            result,
            span: branch_span,
        } = branch;

        // type check the scrutinee
        let typed_scrutinee = check!(
            TypedScrutinee::type_check(scrutinee, namespace, self_type),
            return err(warnings, errors),
            warnings,
            errors
        );

        // calculate the requirements map and the declarations map
        let (match_req_map, match_decl_map) = check!(
            matcher(typed_value, typed_scrutinee, namespace),
            return err(warnings, errors),
            warnings,
            errors
        );

        // create a new namespace for this branch
        let mut namespace = namespace.clone();

        // for every item in the declarations map, create a variable declaration,
        // insert it into the branch namespace, and add it to a block of code statements
        let mut code_block_contents: Vec<TypedAstNode> = vec![];
        for (left_decl, right_decl) in match_decl_map.into_iter() {
            let type_ascription = right_decl.return_type;
            let span = left_decl.span().clone();
            let var_decl = TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                name: left_decl.clone(),
                body: right_decl,
                is_mutable: VariableMutability::Immutable,
                type_ascription,
                const_decl_origin: false,
            });
            namespace.insert_symbol(left_decl, var_decl.clone());
            code_block_contents.push(TypedAstNode {
                content: TypedAstNodeContent::Declaration(var_decl),
                span,
            });
        }

        // type check the branch result
        let typed_result = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: result,
                namespace: &mut namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text,
                self_type,
                mode,
                opts,
            }),
            return err(warnings, errors),
            warnings,
            errors
        );

        // unify the return type from the typed result with the type annotation
        if !typed_result.deterministically_aborts() {
            let (mut new_warnings, new_errors) = unify_with_self(
                typed_result.return_type,
                return_type_annotation,
                self_type,
                &typed_result.span,
                help_text,
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }

        // if the typed branch result is a code block, then add the contents
        // of that code block to the block of code statements that we are already
        // generating. if the typed branch result is not a code block, then add
        // the typed branch result as an ast node to the block of code statements
        let TypedExpression {
            expression: typed_result_expression_variant,
            return_type: typed_result_return_type,
            is_constant: typed_result_is_constant,
            span: typed_result_span,
        } = typed_result;
        match typed_result_expression_variant {
            TypedExpressionVariant::CodeBlock(TypedCodeBlock { mut contents, .. }) => {
                code_block_contents.append(&mut contents);
            }
            typed_result_expression_variant => {
                code_block_contents.push(TypedAstNode {
                    content: TypedAstNodeContent::Expression(TypedExpression {
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
        let new_result = TypedExpression {
            expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                contents: code_block_contents,
                whole_block_span: typed_result_span.clone(),
            }),
            return_type: typed_result.return_type,
            is_constant: IsConstant::No,
            span: typed_result_span,
        };

        // return!
        let branch = TypedMatchBranch {
            conditions: match_req_map,
            result: new_result,
            span: branch_span,
        };
        ok(branch, warnings, errors)
    }
}
