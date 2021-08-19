use core_lang::{AstNode, AstNodeContent, Declaration};

use super::page_type::PageType;

pub(crate) fn traverse_ast_node<'a>(ast_node: &'a AstNode) -> Option<PageType<'a>> {
    match &ast_node.content {
        AstNodeContent::Declaration(dec) => handle_declaration(dec, ast_node),

        _ => None,
    }
}

fn handle_declaration<'a>(dec: &'a Declaration, ast_node: &'a AstNode) -> Option<PageType<'a>> {
    match &dec {
        Declaration::StructDeclaration(struct_dec) => Some(struct_dec.into()),

        _ => None,
    }
}
