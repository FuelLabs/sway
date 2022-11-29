use crate::core::{
    token::{AstToken, SymbolKind, Token, TypedAstToken},
    token_map::TokenMap,
};
use sway_core::{
    declaration_engine,
    language::ty,
    type_system::{TypeId, TypeInfo},
    TypeEngine,
};
use sway_types::{ident::Ident, span::Span, Spanned};

pub fn is_initial_declaration(token_type: &Token) -> bool {
    match &token_type.typed {
        Some(typed_ast_token) => {
            matches!(
                typed_ast_token,
                TypedAstToken::TypedDeclaration(_) | TypedAstToken::TypedFunctionDeclaration(_)
            )
        }
        None => {
            matches!(
                token_type.parsed,
                AstToken::Declaration(_) | AstToken::FunctionDeclaration(_)
            )
        }
    }
}

// Check if the given method is a `core::ops` application desugared from short-hand syntax like / + * - etc.
pub(crate) fn desugared_op(prefixes: &[Ident]) -> bool {
    let prefix0 = prefixes.get(0).map(|ident| ident.as_str());
    let prefix1 = prefixes.get(1).map(|ident| ident.as_str());
    if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
        return true;
    }

    false
}

// We need to do this work around as the custom PartialEq for Ident impl
// only checks for the string, not the span.
pub(crate) fn to_ident_key(ident: &Ident) -> (Ident, Span) {
    (ident.clone(), ident.span())
}

/// Uses the TypeId to find the associated TypedDeclaration in the TokenMap.
pub(crate) fn declaration_of_type_id(
    type_engine: &TypeEngine,
    type_id: &TypeId,
    tokens: &TokenMap,
) -> Option<ty::TyDeclaration> {
    ident_of_type_id(type_engine, type_id)
        .and_then(|decl_ident| tokens.try_get(&to_ident_key(&decl_ident)).try_unwrap())
        .map(|item| item.value().clone())
        .and_then(|token| token.typed)
        .and_then(|typed_token| match typed_token {
            TypedAstToken::TypedDeclaration(dec) => Some(dec),
            _ => None,
        })
}

/// Returns the TypedStructDeclaration associated with the TypeId if it
/// exists within the TokenMap.
pub(crate) fn struct_declaration_of_type_id(
    type_engine: &TypeEngine,
    type_id: &TypeId,
    tokens: &TokenMap,
) -> Option<ty::TyStructDeclaration> {
    declaration_of_type_id(type_engine, type_id, tokens).and_then(|decl| match decl {
        ty::TyDeclaration::StructDeclaration(ref decl_id) => {
            declaration_engine::de_get_struct(decl_id.clone(), &decl_id.span()).ok()
        }
        _ => None,
    })
}

/// Use the TypeId to look up the associated TypeInfo and return the Ident if one is found.
pub(crate) fn ident_of_type_id(type_engine: &TypeEngine, type_id: &TypeId) -> Option<Ident> {
    match type_engine.look_up_type_id(*type_id) {
        TypeInfo::UnknownGeneric { name, .. }
        | TypeInfo::Enum { name, .. }
        | TypeInfo::Struct { name, .. }
        | TypeInfo::Custom { name, .. } => Some(name),
        _ => None,
    }
}

pub(crate) fn type_info_to_symbol_kind(
    type_engine: &TypeEngine,
    type_info: &TypeInfo,
) -> SymbolKind {
    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::Str(..) | TypeInfo::B256 => {
            SymbolKind::BuiltinType
        }
        TypeInfo::Numeric => SymbolKind::NumericLiteral,
        TypeInfo::Custom { .. } | TypeInfo::Struct { .. } => SymbolKind::Struct,
        TypeInfo::Enum { .. } => SymbolKind::Enum,
        TypeInfo::Array(elem_ty, _) => {
            let type_info = type_engine.look_up_type_id(elem_ty.type_id);
            type_info_to_symbol_kind(type_engine, &type_info)
        }
        _ => SymbolKind::Unknown,
    }
}
