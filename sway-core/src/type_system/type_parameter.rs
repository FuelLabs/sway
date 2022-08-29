use crate::{
    error::*,
    semantic_analysis::*,
    type_system::*,
    types::{CompileWrapper, JsonAbiString, ToCompileWrapper, ToJsonAbi},
};

use sway_types::{ident::Ident, span::Span, JsonTypeDeclaration, Spanned};

use std::{
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub(crate) type_id: TypeId,
    pub(crate) initial_type_id: TypeId,
    pub(crate) name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for CompileWrapper<'_, TypeParameter> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        look_up_type_id(me.type_id).wrap(de) == look_up_type_id(them.type_id).wrap(de)
            && me.name_ident == them.name_ident
            && me.trait_constraints == them.trait_constraints
    }
}

impl PartialEq for CompileWrapper<'_, Vec<TypeParameter>> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        if me.len() != them.len() {
            return false;
        }
        me.iter()
            .map(|elem| elem.wrap(de))
            .zip(other.inner.iter().map(|elem| elem.wrap(de)))
            .map(|(left, right)| left == right)
            .all(|elem| elem)
    }
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

impl CopyTypes for TypeParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id = match look_up_type_id(self.type_id).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id, self.name_ident.span())),
            None => {
                let ty = TypeInfo::Ref(insert_type(look_up_type_id_raw(self.type_id)), self.span());
                insert_type(ty)
            }
        };
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

impl ToJsonAbi for TypeParameter {
    type Output = Property;

    fn generate_json_abi(&self) -> Self::Output {
        Property {
            name: self.name_ident.to_string(),
            type_field: self.type_id.json_abi_str(),
            components: self.type_id.generate_json_abi(),
            type_arguments: self
                .type_id
                .get_type_parameters()
                .map(|v| v.iter().map(TypeParameter::generate_json_abi).collect()),
        }
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
