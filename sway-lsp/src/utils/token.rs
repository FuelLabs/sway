use crate::core::token::{AstToken, SymbolKind, Token, TokenMap, TypedAstToken};
use sway_core::declaration_engine;
use sway_core::semantic_analysis::ast_node::{declaration::TyStructDeclaration, TyDeclaration};
use sway_core::type_system::{TypeId, TypeInfo};
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
pub(crate) fn declaration_of_type_id(type_id: &TypeId, tokens: &TokenMap) -> Option<TyDeclaration> {
    ident_of_type_id(type_id)
        .and_then(|decl_ident| tokens.get(&to_ident_key(&decl_ident)))
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
    type_id: &TypeId,
    tokens: &TokenMap,
) -> Option<TyStructDeclaration> {
    declaration_of_type_id(type_id, tokens).and_then(|decl| match decl {
        TyDeclaration::StructDeclaration(ref decl_id) => {
            declaration_engine::de_get_struct(decl_id.clone(), &decl_id.span()).ok()
        }
        _ => None,
    })
}

/// Use the TypeId to look up the associated TypeInfo and return the Ident if one is found.
pub(crate) fn ident_of_type_id(type_id: &TypeId) -> Option<Ident> {
    let type_info = sway_core::type_system::look_up_type_id(*type_id);
    match type_info {
        TypeInfo::UnknownGeneric { name }
        | TypeInfo::Enum { name, .. }
        | TypeInfo::Struct { name, .. }
        | TypeInfo::Custom { name, .. } => Some(name),
        _ => None,
    }
}

pub(crate) fn type_info_to_symbol_kind(type_info: &TypeInfo) -> SymbolKind {
    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::Str(..) | TypeInfo::B256 => {
            SymbolKind::BuiltinType
        }
        TypeInfo::Numeric => SymbolKind::NumericLiteral,
        TypeInfo::Custom { .. } | TypeInfo::Struct { .. } => SymbolKind::Struct,
        TypeInfo::Enum { .. } => SymbolKind::Enum,
        TypeInfo::Array(type_id, ..) => {
            let type_info = sway_core::type_system::look_up_type_id(*type_id);
            type_info_to_symbol_kind(&type_info)
        }
        _ => SymbolKind::Unknown,
    }
}

pub(crate) fn type_id(token_type: &Token) -> Option<TypeId> {
    match &token_type.typed {
        Some(typed_ast_token) => match typed_ast_token {
            TypedAstToken::TypedDeclaration(dec) => match dec {
                TyDeclaration::VariableDeclaration(var_decl) => Some(var_decl.type_ascription),
                TyDeclaration::ConstantDeclaration(decl_id) => {
                    declaration_engine::de_get_constant(decl_id.clone(), &decl_id.span())
                        .ok()
                        .map(|const_decl| const_decl.value.return_type)
                }
                _ => None,
            },
            TypedAstToken::TypedExpression(exp) => Some(exp.return_type),
            TypedAstToken::TypedFunctionParameter(func_param) => Some(func_param.type_id),
            TypedAstToken::TypedStructField(struct_field) => Some(struct_field.type_id),
            TypedAstToken::TypedEnumVariant(enum_var) => Some(enum_var.type_id),
            TypedAstToken::TypedTraitFn(trait_fn) => Some(trait_fn.return_type),
            TypedAstToken::TypedStorageField(storage_field) => Some(storage_field.type_id),
            TypedAstToken::TypeCheckedStorageReassignDescriptor(storage_desc) => {
                Some(storage_desc.type_id)
            }
            TypedAstToken::TypedReassignment(reassignment) => Some(reassignment.lhs_type),
            _ => None,
        },
        None => None,
    }
}
