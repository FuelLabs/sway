use crate::{
    core::token::{AstToken, SymbolKind, Token, TypeDefinition, TypedAstToken},
    traverse::ParseContext,
};
use sway_core::language::{
    parsed::{AstNode, AstNodeContent, Declaration},
    ty,
};
use sway_types::Named;

/// Insert Declaration tokens into the TokenMap.
pub fn collect_parsed_declaration(node: &AstNode, ctx: &ParseContext) {
    if let AstNodeContent::Declaration(declaration) = &node.content {
        let parsed_token = AstToken::Declaration(declaration.clone());

        let (ident, symbol_kind) = match declaration {
            Declaration::VariableDeclaration(decl_id) => {
                let variable = ctx.engines.pe().get_variable(decl_id);
                (variable.name.clone(), SymbolKind::Variable)
            }
            Declaration::StructDeclaration(decl_id) => {
                let decl = ctx.engines.pe().get_struct(decl_id);
                (decl.name.clone(), SymbolKind::Struct)
            }
            Declaration::TraitDeclaration(decl_id) => {
                let decl = ctx.engines.pe().get_trait(decl_id);
                (decl.name.clone(), SymbolKind::Trait)
            }
            Declaration::FunctionDeclaration(decl_id) => {
                let decl = ctx.engines.pe().get_function(decl_id);
                (decl.name.clone(), SymbolKind::Function)
            }
            Declaration::ConstantDeclaration(decl_id) => {
                let decl = ctx.engines.pe().get_constant(decl_id);
                (decl.name.clone(), SymbolKind::Const)
            }
            Declaration::EnumDeclaration(decl_id) => {
                let decl = ctx.engines.pe().get_enum(decl_id);
                (decl.name.clone(), SymbolKind::Enum)
            }
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
            ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id }) => {
                ctx.engines.de().get_constant(decl_id).name().clone()
            }
            ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id }) => {
                ctx.engines.de().get_function(decl_id).name().clone()
            }
            ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id }) => {
                ctx.engines.de().get_trait(decl_id).name().clone()
            }
            ty::TyDecl::StructDecl(ty::StructDecl { decl_id }) => {
                ctx.engines.de().get_struct(decl_id).name().clone()
            }
            ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id }) => {
                ctx.engines.de().get_enum(decl_id).name().clone()
            }
            ty::TyDecl::VariableDecl(variable) => variable.name.clone(),
            _ => return,
        };

        let token_ident = ctx.ident(&ident);
        if let Some(mut token) = ctx.tokens.try_get_mut_with_retry(&token_ident) {
            token.typed = Some(typed_token);
            token.type_def = Some(TypeDefinition::Ident(ident));
        }
    }
}
