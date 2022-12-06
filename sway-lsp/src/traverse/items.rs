use crate::core::{
    token::{to_ident_key, AstToken, SymbolKind, Token},
    token_map::TokenMap,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sway_ast::{AttributeDecl, FnSignature, GenericParams, ItemKind};
use sway_core::TypeEngine;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Span, Spanned};

pub fn parse_module(
    src: Arc<str>,
    path: Arc<PathBuf>,
    tokens: &TokenMap,
) -> Result<(), ErrorEmitted> {
    let handler = <_>::default();
    let module = sway_parse::parse_file(&handler, src, Some(path.clone()))?;

    for item in module.items {
        eprintln!("item = {:#?}", item);
        match &item.value {
            ItemKind::Fn(func) => {
                func.fn_signature.parse(tokens);
            }
            ItemKind::Abi(abi) => for (fn_sig, _) in &abi.abi_items.inner {},
            ItemKind::Impl(item_impl) => for item_fn in &item_impl.contents.inner {},
            _ => (),
        }
    }
    Ok(())
}

pub trait Parse {
    fn parse(&self, tokens: &TokenMap);
}

impl Parse for FnSignature {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = self.visibility {
            let ident = Ident::new(visibility.span());
            let token = Token::from_parsed(AstToken::Keyword(ident), SymbolKind::Keyword);
            tokens.insert(to_ident_key(&ident), token);
        }

        let ident = Ident::new(self.fn_token.span());
        let token = Token::from_parsed(AstToken::Keyword(ident), SymbolKind::Keyword);
        tokens.insert(to_ident_key(&ident.clone()), token);

        let token = Token::from_parsed(AstToken::Ident(self.name), SymbolKind::Unknown);
        tokens.insert(to_ident_key(&ident), token);

        if let Some(generics) = &self.generics {
            generics.parse(tokens);
        }
    }
}

impl Parse for GenericParams {
    fn parse(&self, tokens: &TokenMap) {}
}

// impl Parse for FnArgs {
//     fn parse(&self, tokens: &TokenMap) {}
// }
