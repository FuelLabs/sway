use core_lang::{
    AstNode, AstNodeContent, Declaration, Expression, HllParseTree, ReturnStatement, Span,
};

use crate::traversal_helper::format_struct;

#[derive(Debug)]
pub struct Change {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Change {
    pub fn handle_struct(span: &Span) -> Self {
        Self {
            text: format_struct(span.as_str()),
            start: span.start(),
            end: span.end(),
        }
    }
}

pub fn traverse_for_changes(parse_tree: &HllParseTree) -> Vec<Change> {
    let mut changes = vec![];

    if let Some(script_tree) = &parse_tree.script_ast {
        let nodes = &script_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(&node, &mut changes)
        }
    }

    changes.sort_by(|a, b| a.start.cmp(&b.start));

    changes
}

fn traverse_ast_node(ast_node: &AstNode, changes: &mut Vec<Change>) {
    match &ast_node.content {
        AstNodeContent::Declaration(dec) => handle_declaration(dec, ast_node, changes),

        AstNodeContent::ReturnStatement(ret) => handle_return_statement(ret, ast_node, changes),

        AstNodeContent::Expression(expr) => handle_expression(expr, ast_node, changes),

        AstNodeContent::ImplicitReturnExpression(expr) => {
            handle_implicit_return_expression(expr, ast_node, changes)
        }

        _ => {}
    }
}

fn handle_return_statement(ret: &ReturnStatement, ast_node: &AstNode, changes: &mut Vec<Change>) {
    handle_expression(&ret.expr, ast_node, changes)
}

fn handle_declaration(dec: &Declaration, ast_node: &AstNode, changes: &mut Vec<Change>) {
    match &dec {
        Declaration::VariableDeclaration(var_dec) => {
            handle_expression(&var_dec.body, ast_node, changes)
        }
        Declaration::StructDeclaration(_) => changes.push(Change::handle_struct(&ast_node.span)),

        Declaration::FunctionDeclaration(func) => {
            for content in &func.body.contents {
                traverse_ast_node(&content, changes);
            }
        }
        _ => {}
    };
}

fn handle_expression(expr: &Expression, ast_node: &AstNode, changes: &mut Vec<Change>) {
    match &expr {
        Expression::StructExpression {
            struct_name: _,
            fields: _,
            span: _,
        } => changes.push(Change::handle_struct(&ast_node.span)),
        _ => {}
    }
}

fn handle_implicit_return_expression(
    expr: &Expression,
    ast_node: &AstNode,
    changes: &mut Vec<Change>,
) {
    handle_expression(expr, ast_node, changes)
}
