use crate::error::{err, ok, CompileResult};
use crate::{AstNode, AstNodeContent, ReturnStatement, WhileLoop};
use super::expression::desugar_expression;
use super::declaration::desugar_declaration;

pub fn desugar_ast_node<'sc>(node: AstNode<'sc>) -> CompileResult<'sc, AstNode<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let node = AstNode {
        content: check!(
            desugar_ast_node_content(node.content),
            return err(warnings, errors),
            warnings,
            errors
        ),
        span: node.span
    };
    ok(node, warnings, errors)
}

fn desugar_ast_node_content<'sc>(
    node_content: AstNodeContent<'sc>,
) -> CompileResult<'sc, AstNodeContent<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let node_content = match node_content {
        AstNodeContent::Expression(exp) => {
            AstNodeContent::Expression(check!(
                desugar_expression(exp),
                return err(warnings, errors),
                warnings,
                errors
            ))
        },
        AstNodeContent::Declaration(decl) => {
            AstNodeContent::Declaration(check!(
                desugar_declaration(decl),
                return err(warnings, errors),
                warnings,
                errors
            ))
        },
        AstNodeContent::ImplicitReturnExpression(exp) => {
            AstNodeContent::ImplicitReturnExpression(check!(
                desugar_expression(exp),
                return err(warnings, errors),
                warnings,
                errors
            ))
        },
        AstNodeContent::ReturnStatement(stmt) => {
            AstNodeContent::ReturnStatement(check!(
                desugar_return_stmt(stmt),
                return err(warnings, errors),
                warnings,
                errors
            ))
        },
        AstNodeContent::WhileLoop(while_loop) => {
            AstNodeContent::WhileLoop(check!(
                desugar_while_loop(while_loop),
                return err(warnings, errors),
                warnings,
                errors
            ))
        },
        node_content => node_content.clone()
    };
    ok(node_content, warnings, errors)
}

fn desugar_return_stmt<'sc>(stmt: ReturnStatement<'sc>) -> CompileResult<'sc, ReturnStatement<'sc>> {
    unimplemented!()
}

fn desugar_while_loop<'sc>(while_loop: WhileLoop<'sc>) -> CompileResult<'sc, WhileLoop<'sc>> {
    unimplemented!()
}