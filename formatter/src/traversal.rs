use core_lang::{
    AstNode, AstNodeContent, Declaration, Expression, HllParseTree, ReturnStatement, Span,
};

use crate::traversal_helper::{
    format_custom_types, format_delineated_path, format_include_statement, format_use_statement,
};

#[derive(Debug)]
pub struct Change {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Change {
    fn new(span: &Span, change_type: ChangeType) -> Self {
        let text = match change_type {
            ChangeType::Struct => format_custom_types(span.as_str()),
            ChangeType::Enum => format_custom_types(span.as_str()),
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

pub fn traverse_for_changes(parse_tree: &HllParseTree) -> Vec<Change> {
    let mut changes = vec![];

    if let Some(script_tree) = &parse_tree.script_ast {
        let nodes = &script_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(node, &mut changes)
        }
    }

    if let Some(contract_tree) = &parse_tree.contract_ast {
        let nodes = &contract_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(node, &mut changes)
        }
    }

    if let Some(predicate_tree) = &parse_tree.predicate_ast {
        let nodes = &predicate_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(node, &mut changes)
        }
    }

    for (_, lib_tree) in &parse_tree.library_exports {
        let nodes = &lib_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(node, &mut changes)
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

        AstNodeContent::UseStatement(_) => {
            changes.push(Change::new(&ast_node.span, ChangeType::UseStatement));
        }

        AstNodeContent::IncludeStatement(_) => {
            changes.push(Change::new(&ast_node.span, ChangeType::IncludeStatement))
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

        Declaration::StructDeclaration(_) => {
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
    match &expr {
        Expression::StructExpression {
            struct_name: _,
            fields: _,
            span,
        } => changes.push(Change::new(span, ChangeType::Struct)),
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
                traverse_ast_node(content, changes);
            }
        }
        Expression::DelineatedPath {
            span,
            args: _,
            call_path: _,
            type_arguments: _,
        } => {
            changes.push(Change::new(span, ChangeType::DelineatedPath));
        }
        _ => {}
    }
}

fn handle_implicit_return_expression(expr: &Expression, changes: &mut Vec<Change>) {
    handle_expression(expr, changes)
}
