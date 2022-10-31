use crate::{
    core::token::{SymbolKind, Token, TokenMap, TypedAstToken},
    utils::token::to_ident_key,
};
use sway_core::language::ty;
use sway_types::ident::Ident;

pub fn handle_declaration(ident: &Ident, declaration: &ty::TyDeclaration, tokens: &TokenMap) {
    let key = to_ident_key(ident);
    let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

    match declaration {
        ty::TyDeclaration::VariableDeclaration(_) => {
            tokens.insert(key, Token::from_typed(typed_token, SymbolKind::Variable));
        }
        ty::TyDeclaration::StructDeclaration(_) => {
            tokens.insert(key, Token::from_typed(typed_token, SymbolKind::Struct));
        }
        ty::TyDeclaration::TraitDeclaration(_) => {
            tokens.insert(key, Token::from_typed(typed_token, SymbolKind::Trait));
        }
        ty::TyDeclaration::FunctionDeclaration(_) => {
            tokens.insert(key, Token::from_typed(typed_token, SymbolKind::Function));
        }
        ty::TyDeclaration::ConstantDeclaration(_) => {
            tokens.insert(key, Token::from_typed(typed_token, SymbolKind::Const));
        }
        ty::TyDeclaration::EnumDeclaration(_) => {
            tokens.insert(key, Token::from_typed(typed_token, SymbolKind::Enum));
        }
        _ => {}
    }
}
