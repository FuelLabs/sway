use crate::{
    core::token::{AstToken, SymbolKind, Token, TokenMap, TypeDefinition, TypedAstToken},
    utils::token::to_ident_key,
};
use sway_core::{
    language::{
        parsed::{AstNode, AstNodeContent, Declaration},
        ty,
    },
    Engines,
};
use sway_types::Spanned;

/// Insert Declaration tokens into the TokenMap.
pub fn collect_parsed_declaration(node: &AstNode, tokens: &TokenMap) {
    if let AstNodeContent::Declaration(declaration) = &node.content {
        let parsed_token = AstToken::Declaration(declaration.clone());

        let (ident, symbol_kind) = match declaration {
            Declaration::VariableDeclaration(variable) => {
                (variable.name.clone(), SymbolKind::Variable)
            }
            Declaration::StructDeclaration(decl) => (decl.name.clone(), SymbolKind::Struct),
            Declaration::TraitDeclaration(decl) => (decl.name.clone(), SymbolKind::Trait),
            Declaration::FunctionDeclaration(decl) => (decl.name.clone(), SymbolKind::Function),
            Declaration::ConstantDeclaration(decl) => (decl.name.clone(), SymbolKind::Const),
            Declaration::EnumDeclaration(decl) => (decl.name.clone(), SymbolKind::Enum),
            _ => return,
        };

        let key = to_ident_key(&ident);
        let token = Token::from_parsed(parsed_token, symbol_kind);
        tokens.insert(key, token);
    }
}

/// Insert TypedDeclaration tokens into the TokenMap.
pub fn collect_typed_declaration(engines: Engines<'_>, node: &ty::TyAstNode, tokens: &TokenMap) {
    let declaration_engine = engines.de();
    if let ty::TyAstNodeContent::Declaration(declaration) = &node.content {
        let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

        if let Ok(ident) = match declaration {
            ty::TyDeclaration::VariableDeclaration(variable) => Ok(variable.name.clone()),
            ty::TyDeclaration::StructDeclaration(decl_id) => declaration_engine
                .get_struct(decl_id.clone(), &declaration.span())
                .map(|decl| decl.name),
            ty::TyDeclaration::TraitDeclaration(decl_id) => declaration_engine
                .get_trait(decl_id.clone(), &declaration.span())
                .map(|decl| decl.name),
            ty::TyDeclaration::FunctionDeclaration(decl_id) => declaration_engine
                .get_function(decl_id.clone(), &declaration.span())
                .map(|decl| decl.name),
            ty::TyDeclaration::ConstantDeclaration(decl_id) => declaration_engine
                .get_constant(decl_id.clone(), &declaration.span())
                .map(|decl| decl.name),
            ty::TyDeclaration::EnumDeclaration(decl_id) => declaration_engine
                .get_enum(decl_id.clone(), &declaration.span())
                .map(|decl| decl.name),
            _ => return,
        } {
            let ident = to_ident_key(&ident);
            if let Some(mut token) = tokens.get_mut(&ident) {
                token.typed = Some(typed_token);
                token.type_def = Some(TypeDefinition::Ident(ident.0));
            }
        }
    }
}
