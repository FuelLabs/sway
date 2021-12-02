use crate::error::{err, ok, CompileResult};
use crate::{AstNode, AstNodeContent};
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
    match node_content {
        AstNodeContent::Expression(exp) => {
            let node_content = AstNodeContent::Expression(check!(
                desugar_expression(exp),
                return err(warnings, errors),
                warnings,
                errors
            ));
            ok(node_content, warnings, errors)
        },
        AstNodeContent::Declaration(decl) => {
            let node_content = AstNodeContent::Declaration(check!(
                desugar_declaration(decl),
                return err(warnings, errors),
                warnings,
                errors
            ));
            ok(node_content, warnings, errors)
        },
        AstNodeContent::ImplicitReturnExpression(exp) => {
            let node_content = AstNodeContent::ImplicitReturnExpression(check!(
                desugar_expression(exp),
                return err(warnings, errors),
                warnings,
                errors
            ));
            ok(node_content, warnings, errors)
        },
        node => unimplemented!("{:?}", node)
    }
}
