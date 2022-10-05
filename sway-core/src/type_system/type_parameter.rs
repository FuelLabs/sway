use crate::{error::*, semantic_analysis::*, type_system::*};

use sway_types::{ident::Ident, span::Span, JsonTypeDeclaration, Spanned};

use std::{
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone, Eq)]
pub struct TypeParameter {
    pub type_id: TypeId,
    pub(crate) initial_type_id: TypeId,
    pub name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        look_up_type_id(self.type_id).hash(state);
        self.name_ident.hash(state);
        self.trait_constraints.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeParameter {
    fn eq(&self, other: &Self) -> bool {
        look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.name_ident == other.name_ident
            && self.trait_constraints == other.trait_constraints
    }
}

impl CopyTypes for TypeParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl ReplaceSelfType for TypeParameter {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
    }
}

impl fmt::Display for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name_ident, self.type_id)
    }
}

impl fmt::Debug for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name_ident, self.type_id)
    }
}

impl TypeParameter {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        type_parameter: TypeParameter,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if !type_parameter.trait_constraints.is_empty() {
            errors.push(CompileError::WhereClauseNotYetSupported {
                span: type_parameter.name_ident.span(),
            });
            return err(warnings, errors);
        }
        // TODO: add check here to see if the type parameter has a valid name and does not have type parameters
        let type_id = insert_type(TypeInfo::UnknownGeneric {
            name: type_parameter.name_ident.clone(),
        });
        let type_parameter_decl = TypedDeclaration::GenericTypeForFunctionScope {
            name: type_parameter.name_ident.clone(),
            type_id,
        };
        ctx.namespace
            .insert_symbol(type_parameter.name_ident.clone(), type_parameter_decl)
            .ok(&mut warnings, &mut errors);
        let type_parameter = TypeParameter {
            name_ident: type_parameter.name_ident,
            type_id,
            initial_type_id: type_parameter.initial_type_id,
            trait_constraints: type_parameter.trait_constraints,
        };
        ok(type_parameter, warnings, errors)
    }

    /// Returns the initial type ID of a TypeParameter. Also updates the provided list of types to
    /// append the current TypeParameter as a `JsonTypeDeclaration`.
    pub(crate) fn get_json_type_parameter(&self, types: &mut Vec<JsonTypeDeclaration>) -> usize {
        let type_parameter = JsonTypeDeclaration {
            type_id: *self.initial_type_id,
            type_field: self.initial_type_id.get_json_type_str(self.type_id),
            components: self
                .initial_type_id
                .get_json_type_components(types, self.type_id),
            type_parameters: None,
        };
        types.push(type_parameter);
        *self.initial_type_id
    }
}
