use crate::{
    core::token::{SymbolKind, Token, TokenMap, TypeDefinition, TypedAstToken},
    utils::token::{struct_declaration_of_type_id, to_ident_key},
};
use sway_core::{
    declaration_engine::{self, de_get_function},
    language::ty,
};
use sway_types::constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX};
use sway_types::{ident::Ident, Spanned};

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
