use crate::{
    core::token::{AstToken, SymbolKind, Token, TokenMap},
    utils::token::to_ident_key,
};
use std::sync::Arc;
use sway_ast::{AttributeDecl, ItemKind};
use sway_core::BuildConfig;

pub(crate) fn parse(src: Arc<str>, build_config: Option<&BuildConfig>, tokens: &TokenMap) {
    let path = build_config.map(|build_config| build_config.canonical_root_module());
    if let Ok(module) = sway_parse::parse_file_standalone(src, path) {
        for item in module.items {
            match &item.value {
                ItemKind::Abi(abi) => {
                    for (fn_sig, _) in &abi.abi_items.inner {
                        collect_storage_attribute(&fn_sig.attribute_list, tokens);
                    }
                }
                ItemKind::Impl(item_impl) => {
                    for item_fn in &item_impl.contents.inner {
                        collect_storage_attribute(&item_fn.attribute_list, tokens);
                    }
                }
                _ => (),
            }
        }
    }
}

fn collect_storage_attribute(attribute_list: &Vec<AttributeDecl>, tokens: &TokenMap) {
    for attribute_decl in attribute_list {
        let ident = attribute_decl.attribute.inner.name.clone();
        let token = Token::from_parsed(AstToken::Keyword(ident.clone()), SymbolKind::BuiltinAttr);
        tokens.insert(to_ident_key(&ident), token.clone());
    }
}
