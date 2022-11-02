use crate::{
    core::token::{SymbolKind, Token, TokenMap, TypedAstToken},
    utils::token::to_ident_key,
};
use sway_core::language::ty;
use sway_types::ident::Ident;

/// Insert TypedDeclaration tokens into the TokenMap.
pub fn handle_declaration(ident: &Ident, declaration: &ty::TyDeclaration, tokens: &TokenMap) {
    let key = to_ident_key(ident);
    let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

    let symbol_kind = match declaration {
        ty::TyDeclaration::VariableDeclaration(_) => SymbolKind::Variable,
        ty::TyDeclaration::StructDeclaration(_) => SymbolKind::Struct,
        ty::TyDeclaration::TraitDeclaration(_) => SymbolKind::Trait,
        ty::TyDeclaration::FunctionDeclaration(_) => SymbolKind::Function,
        ty::TyDeclaration::ConstantDeclaration(_) => SymbolKind::Const,
        ty::TyDeclaration::EnumDeclaration(_) => SymbolKind::Enum,
        _ => return,
    };

    tokens.insert(key, Token::from_typed(typed_token, symbol_kind));
}
