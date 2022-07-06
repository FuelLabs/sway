use crate::core::token::{AstToken, TokenType, TypedAstToken};
use sway_core::semantic_analysis::ast_node::TypedDeclaration;
use sway_core::{parse_tree::MethodName, type_engine::TypeId};
use sway_types::{ident::Ident, span::Span, Spanned};


pub fn is_initial_declaration(token_type: &TokenType) -> bool {
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
pub(crate) fn desugared_op(method_name: &MethodName) -> bool {
    if let MethodName::FromType { ref call_path, .. } | MethodName::FromTrait { ref call_path } =
        method_name
    {
        let prefix0 = call_path.prefixes.get(0).map(|ident| ident.as_str());
        let prefix1 = call_path.prefixes.get(1).map(|ident| ident.as_str());
        if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
            return true;
        }
    }
    false
}

// We need to do this work around as the custom PartialEq for Ident impl
// only checks for the string, not the span.
pub(crate) fn to_ident_key(ident: &Ident) -> (Ident, Span) {
    (ident.clone(), ident.span())
}

pub fn get_type_id(token_type: &TokenType) -> Option<TypeId> {
    match &token_type.typed {
        Some(typed_ast_token) => match typed_ast_token {
            TypedAstToken::TypedDeclaration(dec) => match dec {
                TypedDeclaration::VariableDeclaration(var_decl) => Some(var_decl.type_ascription),
                TypedDeclaration::ConstantDeclaration(const_decl) => {
                    Some(const_decl.value.return_type)
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
