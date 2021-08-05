use core_lang::{AstNode, AstNodeContent, Declaration, HllParseTree};

use super::builder;

pub fn traverse_and_build(parse_tree: HllParseTree) -> Result<(), String> {
    if let Some(script_tree) = &parse_tree.script_ast {
        let nodes = &script_tree.root_nodes;

        for node in nodes {
            traverse_ast_node(&node);
        }
    }

    Ok(())
}

fn traverse_ast_node(ast_node: &AstNode) {
    match &ast_node.content {
        AstNodeContent::Declaration(dec) => handle_declaration(dec, ast_node),

        _ => {}
    }
}

fn handle_declaration(dec: &Declaration, ast_node: &AstNode) {
    match &dec {
        Declaration::StructDeclaration(struct_dec) => builder::build_struct(struct_dec),

        _ => {}
    };
}
