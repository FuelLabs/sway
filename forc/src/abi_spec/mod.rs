mod declaration;

use core_lang::semantic_analysis::ast_node::{TypedAstNode, TypedAstNodeContent};
use core_lang::semantic_analysis::ast_node::{
    TypedExpression, TypedReturnStatement, TypedWhileLoop,
};
use core_lang::semantic_analysis::TypedParseTree;
use core_lang::CompileResult;

use serde_json::Value;

pub fn generate_abi_spec<'sc>(tree: TypedParseTree<'sc>) -> CompileResult<'sc, Value> {
    let all_nodes = match tree {
        TypedParseTree::Contract { all_nodes, .. } => all_nodes,
        TypedParseTree::Library { all_nodes, .. } => all_nodes,
        TypedParseTree::Predicate { all_nodes, .. } => all_nodes,
        TypedParseTree::Script { all_nodes, .. } => all_nodes,
    };

    for node in all_nodes {
        generate_abi_spec_node(node);
    }

    unimplemented!()
}

fn generate_abi_spec_node<'sc>(node: TypedAstNode<'sc>) {
    generate_abi_spec_node_content(node.content)
}

fn generate_abi_spec_node_content<'sc>(node_content: TypedAstNodeContent<'sc>) {
    match node_content {
        TypedAstNodeContent::Declaration(declaration) => {
            declaration::generate_abi_spec(declaration)
        }
        TypedAstNodeContent::Expression(expression) => generate_abi_spec_expression(expression),
        TypedAstNodeContent::ReturnStatement(return_statement) => {
            generate_abi_spec_return_statement(return_statement)
        }
        TypedAstNodeContent::ImplicitReturnExpression(implicit_return_statement) => {
            generate_abi_spec_implicit_return_statement(implicit_return_statement)
        }
        TypedAstNodeContent::WhileLoop(while_loop) => generate_abi_spec_while_loop(while_loop),
        TypedAstNodeContent::SideEffect => {}
    }
}

fn generate_abi_spec_expression<'sc>(expression: TypedExpression<'sc>) {
    // todo
}

fn generate_abi_spec_return_statement<'sc>(return_statement: TypedReturnStatement<'sc>) {
    // todo
}

fn generate_abi_spec_implicit_return_statement<'sc>(
    implicit_return_statement: TypedExpression<'sc>,
) {
    // todo
}

fn generate_abi_spec_while_loop<'sc>(while_loop: TypedWhileLoop<'sc>) {
    // todo
}
