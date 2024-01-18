use crate::{
    core::token::{AstToken, SymbolKind, Token, TypeDefinition, TypedAstToken},
    traverse::ParseContext,
};
use sway_core::language::{
    parsed::{AstNode, AstNodeContent, Declaration},
    ty,
};

/// Insert Declaration tokens into the TokenMap.
pub fn collect_parsed_declaration(node: &AstNode, ctx: &ParseContext) {
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

        let token = Token::from_parsed(parsed_token, symbol_kind);
        ctx.tokens.insert(ctx.ident(&ident), token);
    }
}

/// Insert TypedDeclaration tokens into the TokenMap.
pub fn collect_typed_declaration(node: &ty::TyAstNode, ctx: &ParseContext) {
    if let ty::TyAstNodeContent::Declaration(declaration) = &node.content {
        let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

        let ident = match declaration {
            ty::TyDecl::VariableDecl(variable) => variable.name.clone(),
            ty::TyDecl::StructDecl(ty::StructDecl { name, .. })
            | ty::TyDecl::EnumDecl(ty::EnumDecl { name, .. })
            | ty::TyDecl::TraitDecl(ty::TraitDecl { name, .. })
            | ty::TyDecl::FunctionDecl(ty::FunctionDecl { name, .. })
            | ty::TyDecl::ConstantDecl(ty::ConstantDecl { name, .. }) => name.clone(),
            _ => return,
        };

        let token_ident = ctx.ident(&ident);
        if let Some(mut token) = ctx.tokens.try_get_mut_with_retry(&token_ident) {
            token.typed = Some(typed_token);
            token.type_def = Some(TypeDefinition::Ident(ident));
        }
    }
}
