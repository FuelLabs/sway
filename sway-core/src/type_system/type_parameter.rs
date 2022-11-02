use crate::{error::*, language::ty, semantic_analysis::*, type_system::*};

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
    pub(crate) trait_constraints_span: Span,
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
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl ReplaceSelfType for TypeParameter {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.type_id.replace_self_type(self_type);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.replace_self_type(self_type));
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
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
        mut ctx: TypeCheckContext,
        type_parameter: TypeParameter,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TypeParameter {
            initial_type_id,
            name_ident,
            mut trait_constraints,
            trait_constraints_span,
            ..
        } = type_parameter;

        // Type check the trait constraints.
        for trait_constraint in trait_constraints.iter_mut() {
            check!(
                trait_constraint.type_check(ctx.by_ref()),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        // TODO: add check here to see if the type parameter has a valid name and does not have type parameters

        let type_id = insert_type(TypeInfo::UnknownGeneric {
            name: name_ident.clone(),
            trait_constraints: trait_constraints.clone().into_iter().collect(),
        });

        // Insert the trait constraints into the namespace.
        // We insert this type with it's own copy of the type info so that as
        // types resolve in the type engine, the implemented traits for the
        // trait constraints don't inadvertently point to the resolved type for
        // the resolved type parameter.
        for trait_constraint in trait_constraints.iter() {
            check!(
                TraitConstraint::insert_into_namespace(ctx.by_ref(), type_id, trait_constraint),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        // Insert the type parameter into the namespace as a dummy type
        // declaration.
        let type_parameter_decl = ty::TyDeclaration::GenericTypeForFunctionScope {
            name: name_ident.clone(),
            type_id,
        };
        ctx.namespace
            .insert_symbol(name_ident.clone(), type_parameter_decl)
            .ok(&mut warnings, &mut errors);

        let type_parameter = TypeParameter {
            name_ident,
            type_id,
            initial_type_id,
            trait_constraints,
            trait_constraints_span,
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
