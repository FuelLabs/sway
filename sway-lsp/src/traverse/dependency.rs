use crate::core::{
    token::{self, AstToken, SymbolKind, Token, TypeDefinition, TypedAstToken},
    token_map::TokenMap,
};
use sway_core::{
    decl_engine as de,
    language::{
        parsed::{AstNode, AstNodeContent, Declaration},
        ty,
    },
};
use sway_types::Spanned;

pub struct Dependency<'a> {
    tokens: &'a TokenMap,
}

impl<'a> Dependency<'a> {
    pub fn new(tokens: &'a TokenMap) -> Self {
        Self { tokens }
    }

    /// Insert Declaration tokens into the TokenMap.
    pub fn collect_parsed_declaration(&self, node: &AstNode) {
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

            let key = token::to_ident_key(&ident);
            let token = Token::from_parsed(parsed_token, symbol_kind);
            self.tokens.insert(key, token);
        }
    }

    /// Insert TypedDeclaration tokens into the TokenMap.
    pub fn collect_typed_declaration(&self, decl_engine: &de::DeclEngine, node: &ty::TyAstNode) {
        if let ty::TyAstNodeContent::Declaration(declaration) = &node.content {
            let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());

            if let Ok(ident) = match declaration {
                ty::TyDeclaration::VariableDeclaration(variable) => Ok(variable.name.clone()),
                ty::TyDeclaration::StructDeclaration(decl_id) => decl_engine
                    .get_struct(decl_id.clone(), &declaration.span())
                    .map(|decl| decl.name),
                ty::TyDeclaration::TraitDeclaration(decl_id) => decl_engine
                    .get_trait(decl_id.clone(), &declaration.span())
                    .map(|decl| decl.name),
                ty::TyDeclaration::FunctionDeclaration(decl_id) => decl_engine
                    .get_function(decl_id.clone(), &declaration.span())
                    .map(|decl| decl.name),
                ty::TyDeclaration::ConstantDeclaration(decl_id) => decl_engine
                    .get_constant(decl_id.clone(), &declaration.span())
                    .map(|decl| decl.name),
                ty::TyDeclaration::EnumDeclaration(decl_id) => decl_engine
                    .get_enum(decl_id.clone(), &declaration.span())
                    .map(|decl| decl.name),
                _ => return,
            } {
                let ident = token::to_ident_key(&ident);
                if let Some(mut token) = self.tokens.try_get_mut(&ident).try_unwrap() {
                    token.typed = Some(typed_token);
                    token.type_def = Some(TypeDefinition::Ident(ident.0));
                }
            }
        }
    }
}
