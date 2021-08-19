use core_lang::{AstNode, AstNodeContent, Declaration, HllParseTree};

use super::page_type::PageType;

pub(crate) fn traverse_for_page_types(parse_tree: HllParseTree) -> Vec<PageType> {
    let mut res = vec![];

    if let Some(script_tree) = &parse_tree.script_ast {
        let nodes = &script_tree.root_nodes;

        for node in nodes {
            if let Some(page_type) = traverse_ast_node(&node) {
                res.push(page_type);
            }
        }
    }

    res
}

fn traverse_ast_node(ast_node: &AstNode) -> Option<PageType> {
    match &ast_node.content {
        AstNodeContent::Declaration(dec) => handle_declaration(dec, ast_node),

        _ => None,
    }
}

fn handle_declaration(dec: &Declaration, ast_node: &AstNode) -> Option<PageType> {
    match &dec {
        Declaration::StructDeclaration(struct_dec) => Some(struct_dec.into()),
        _ => None,
    }
}
