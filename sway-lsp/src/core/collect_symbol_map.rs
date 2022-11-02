use crate::{
    core::token::{SymbolKind, Token, TokenMap, TypedAstToken},
    utils::token::to_ident_key,
};
use sway_core::{declaration_engine as de, language::ty};
use sway_types::{Ident, Spanned};

/// Insert TypedDeclaration tokens into the TokenMap.
// pub fn handle_declaration(ident: &Ident, declaration: &ty::TyDeclaration, tokens: &TokenMap) {
//     let key = to_ident_key(ident);
//     let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

//     let symbol_kind = match declaration {
//         ty::TyDeclaration::VariableDeclaration(_) => SymbolKind::Variable,
//         ty::TyDeclaration::StructDeclaration(_) => SymbolKind::Struct,
//         ty::TyDeclaration::TraitDeclaration(_) => SymbolKind::Trait,
//         ty::TyDeclaration::FunctionDeclaration(_) => SymbolKind::Function,
//         ty::TyDeclaration::ConstantDeclaration(_) => SymbolKind::Const,
//         ty::TyDeclaration::EnumDeclaration(_) => SymbolKind::Enum,
//         _ => return,
//     };

//     tokens.insert(key, Token::from_typed(typed_token, symbol_kind));
// }

/// Insert TypedDeclaration tokens into the TokenMap.
pub fn collect_declaration(node: &ty::TyAstNode, tokens: &TokenMap) {
    if let ty::TyAstNodeContent::Declaration(declaration) = &node.content {
        let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

        if let Ok((ident, symbol_kind)) = match declaration {
            ty::TyDeclaration::VariableDeclaration(variable) => {
                Ok((variable.name.clone(), SymbolKind::Variable))
            }
            ty::TyDeclaration::StructDeclaration(decl_id) => {
                de::de_get_struct(decl_id.clone(), &declaration.span())
                    .map(|decl| (decl.name, SymbolKind::Struct))
            }
            ty::TyDeclaration::TraitDeclaration(decl_id) => {
                de::de_get_trait(decl_id.clone(), &declaration.span())
                    .map(|decl| (decl.name, SymbolKind::Trait))
            }
            ty::TyDeclaration::FunctionDeclaration(decl_id) => {
                de::de_get_function(decl_id.clone(), &declaration.span())
                    .map(|decl| (decl.name, SymbolKind::Function))
            }
            ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                de::de_get_constant(decl_id.clone(), &declaration.span())
                    .map(|decl| (decl.name, SymbolKind::Const))
            }
            ty::TyDeclaration::EnumDeclaration(decl_id) => {
                de::de_get_enum(decl_id.clone(), &declaration.span())
                    .map(|decl| (decl.name, SymbolKind::Enum))
            }
            _ => return,
        } {
            let key = to_ident_key(&ident);
            let token = Token::from_typed(typed_token, symbol_kind);
            tokens.insert(key, token);
        }
    }
}
