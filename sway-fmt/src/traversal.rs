use sway_core::{
    AstNode, AstNodeContent, Declaration, Expression, ExpressionKind, IfExpression, ParseTree,
    ReturnStatement,
};

use sway_types::span::Span;

use crate::traversal_helper::{
    format_data_types, format_delineated_path, format_include_statement, format_use_statement,
};

/// Change contains the formatted change itself.
/// `start` and `end` denote the start and end of that change,
/// which are used to caluclate the position for inserting this change in the existing file.
#[derive(Debug)]
pub struct Change {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Change {
    fn new(span: &Span, change_type: ChangeType) -> Self {
        let text = match change_type {
            ChangeType::Struct => format_data_types(span.as_str()),
            ChangeType::Enum => format_data_types(span.as_str()),
            ChangeType::IncludeStatement => format_include_statement(span.as_str()),
            ChangeType::UseStatement => format_use_statement(span.as_str()),
            ChangeType::DelineatedPath => format_delineated_path(span.as_str()),
        };

        Self {
            text,
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Debug)]
enum ChangeType {
    Struct,
    Enum,
    IncludeStatement,
    UseStatement,
    DelineatedPath,
}

/// traverses the Sway ParseTree and returns list of formatted changes
pub fn traverse_for_changes(parse_tree: &ParseTree) -> Vec<Change> {
    let mut changes = vec![];

    for node in &parse_tree.root_nodes {
        traverse_ast_node(node, &mut changes);
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

        AstNodeContent::UseStatement(_) => {
            // The AST generates one root node per use statement, we must avoid duplicating them
            // while formatting
            let next_span = &ast_node.span;
            match changes.last() {
                Some(last_change) => {
                    if last_change.start != next_span.start() {
                        changes.push(Change::new(next_span, ChangeType::UseStatement));
                    }
                }
                _ => changes.push(Change::new(next_span, ChangeType::UseStatement)),
            }
        }

        AstNodeContent::IncludeStatement(_) => {
            changes.push(Change::new(&ast_node.span, ChangeType::IncludeStatement))
        }
    }
}

fn handle_return_statement(ret: &ReturnStatement, changes: &mut Vec<Change>) {
    handle_expression(&ret.expr, changes)
}

fn handle_declaration(dec: &Declaration, ast_node: &AstNode, changes: &mut Vec<Change>) {
    match &dec {
        Declaration::VariableDeclaration(var_dec) => handle_expression(&var_dec.body, changes),

        Declaration::StructDeclaration(_) | Declaration::StorageDeclaration(_) => {
            changes.push(Change::new(&ast_node.span, ChangeType::Struct))
        }

        Declaration::EnumDeclaration(_) => {
            changes.push(Change::new(&ast_node.span, ChangeType::Enum))
        }

        Declaration::FunctionDeclaration(func) => {
            for content in &func.body.contents {
                traverse_ast_node(content, changes);
            }
        }

        Declaration::ImplSelf(impl_self) => {
            for func in &impl_self.functions {
                for content in &func.body.contents {
                    traverse_ast_node(content, changes);
                }
            }
        }
        Declaration::ImplTrait(impl_trait) => {
            for func in &impl_trait.functions {
                for content in &func.body.contents {
                    traverse_ast_node(content, changes);
                }
            }
        }
        _ => {}
    };
}

fn handle_expression(expr: &Expression, changes: &mut Vec<Change>) {
    let span = &expr.span;
    match &expr.kind {
        ExpressionKind::Struct(_) => changes.push(Change::new(span, ChangeType::Struct)),
        ExpressionKind::If(IfExpression {
            condition: _,
            then,
            r#else,
        }) => {
            handle_expression(then, changes);

            if let Some(else_expr) = r#else {
                handle_expression(else_expr, changes);
            }
        }
        ExpressionKind::CodeBlock(contents) => {
            for content in &contents.contents {
                traverse_ast_node(content, changes);
            }
        }
        ExpressionKind::DelineatedPath(_) => {
            changes.push(Change::new(span, ChangeType::DelineatedPath));
        }
        _ => {}
    }
}

fn handle_implicit_return_expression(expr: &Expression, changes: &mut Vec<Change>) {
    handle_expression(expr, changes)
}
