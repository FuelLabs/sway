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

    if let Some(contract_tree) = &parse_tree.contract_ast {
        let nodes = &contract_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(&node, &mut changes)
        }
    }

    if let Some(predicate_tree) = &parse_tree.predicate_ast {
        let nodes = &predicate_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(&node, &mut changes)
        }
    }

    for (_, lib_tree) in &parse_tree.library_exports {
        let nodes = &lib_tree.root_nodes;

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

        AstNodeContent::ReturnStatement(ret) => handle_return_statement(ret, changes),

        AstNodeContent::Expression(expr) => handle_expression(expr, changes),

        AstNodeContent::ImplicitReturnExpression(expr) => {
            handle_implicit_return_expression(expr, changes)
        }

        _ => {}
    }
}

fn handle_return_statement(ret: &ReturnStatement, changes: &mut Vec<Change>) {
    handle_expression(&ret.expr, changes)
}

fn handle_declaration(dec: &Declaration, ast_node: &AstNode, changes: &mut Vec<Change>) {
    match &dec {
        Declaration::VariableDeclaration(var_dec) => handle_expression(&var_dec.body, changes),

        Declaration::StructDeclaration(_) => changes.push(Change::handle_struct(&ast_node.span)),

        Declaration::FunctionDeclaration(func) => {
            for content in &func.body.contents {
                traverse_ast_node(&content, changes);
            }
        }
        _ => {}
    };
}

fn handle_expression(expr: &Expression, changes: &mut Vec<Change>) {
    match &expr {
        Expression::StructExpression {
            struct_name: _,
            fields: _,
            span,
        } => changes.push(Change::handle_struct(span)),
        Expression::IfExp {
            condition: _,
            then,
            r#else,
            span: _,
        } => {
            handle_expression(then, changes);

            if let Some(else_expr) = r#else {
                handle_expression(else_expr, changes);
            }
        }
        Expression::CodeBlock { contents, span: _ } => {
            for content in &contents.contents {
                traverse_ast_node(&content, changes);
            }
        }
        _ => {}
    }
}

fn handle_implicit_return_expression(expr: &Expression, changes: &mut Vec<Change>) {
    handle_expression(expr, changes)
}
