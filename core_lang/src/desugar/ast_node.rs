use crate::error::{err, ok, CompileResult};
use crate::{AstNode, AstNodeContent, ReturnStatement, WhileLoop};
use super::code_block::desugar_code_block;
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
    let warnings = vec!();
    let errors = vec!();
    let stmt = ReturnStatement {
        expr: check!(
            desugar_expression(stmt.expr),
            return err(warnings, errors),
            warnings,
            errors
        )
    };
    ok(stmt, warnings, errors)
}

fn desugar_while_loop<'sc>(while_loop: WhileLoop<'sc>) -> CompileResult<'sc, WhileLoop<'sc>> {
    let warnings = vec![];
    let errors = vec![];
    let stmt = WhileLoop {
        condition: check!(
            desugar_expression(while_loop.condition),
            return err(warnings, errors),
            warnings,
            errors
        ),
        body: check!(
            desugar_code_block(while_loop.body),
            return err(warnings, errors),
            warnings,
            errors
        )
    };
    ok(stmt, warnings, errors)
}