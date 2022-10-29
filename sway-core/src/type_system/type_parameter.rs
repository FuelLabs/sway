use crate::{
    declaration_engine::{de_get_trait, de_get_trait_fn, de_insert_function},
    error::*,
    language::ty,
    semantic_analysis::*,
    type_system::*,
};

use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, JsonTypeDeclaration, Spanned};

use std::{
    collections::BTreeSet,
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
        mut ctx: TypeCheckContext,
        type_parameter: TypeParameter,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TypeParameter {
            type_id,
            initial_type_id,
            name_ident,
            mut trait_constraints,
            trait_constraints_span,
        } = type_parameter;

        // type check the trait constraints
        for trait_constraint in trait_constraints.iter_mut() {
            check!(
                trait_constraint.type_check(ctx.by_ref()),
                return err(warnings, errors),
                warnings,
                errors
            );
        }
        let trait_constraints = trait_constraints.into_iter().collect::<BTreeSet<_>>();

        // TODO: add check here to see if the type parameter has a valid name and does not have type parameters

        // create a new type id with the trait constraints added
        let type_id = insert_type(TypeInfo::UnknownGeneric {
            name: name_ident.clone(),
            trait_constraints,
        });

        // insert the generic type into the namespace as a dummy declaration
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

fn insert_trait_constraints(
    mut ctx: TypeCheckContext,
    type_id: TypeId,
    trait_constraints: BTreeSet<TraitConstraint>,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let self_type = ctx.self_type();

    for trait_constraint in trait_constraints.into_iter() {
        match ctx
            .namespace
            .resolve_call_path(&trait_constraint.trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let ty::TyTraitDeclaration {
                    ref interface_surface,
                    ref methods,
                    ref supertraits,
                    ref name,
                    ref type_parameters,
                    ..
                } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &trait_constraint.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // transform the interface surface into methods
                let mut trait_methods = vec![];
                for decl_id in interface_surface {
                    let trait_fn = check!(
                        CompileResult::from(de_get_trait_fn(decl_id.clone(), &name.span())),
                        continue,
                        warnings,
                        errors
                    );
                    trait_methods.push(de_insert_function(trait_fn.to_dummy_func(Mode::NonAbi)));
                }

                // insert dummy versions of the interfaces for all of the supertraits
                // specifically don't check for conflicting definitions because
                // these are just dummy definitions
                ctx.namespace.insert_trait_implementation(
                    trait_constraint.trait_name,
                    vec![],
                    self_type,
                    &trait_methods,
                    &trait_constraint.span(),
                );
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: trait_constraint.trait_name.to_string(),
                span: trait_constraint.trait_name.span(),
            }),
        }
    }

    todo!();
}

/// Recursively handle supertraits by adding all their interfaces and methods to some namespace
/// which is meant to be the namespace of the subtrait in question
fn handle_supertraits(mut ctx: TypeCheckContext, supertraits: &[Supertrait]) -> CompileResult<()> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let self_type = ctx.self_type();

    for supertrait in supertraits.iter() {
        match ctx
            .namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let ty::TyTraitDeclaration {
                    ref interface_surface,
                    ref methods,
                    ref supertraits,
                    ref name,
                    ref type_parameters,
                    ..
                } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &supertrait.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // transform the interface surface into methods
                let mut trait_methods = vec![];
                for decl_id in interface_surface {
                    let trait_fn = check!(
                        CompileResult::from(de_get_trait_fn(decl_id.clone(), &name.span())),
                        continue,
                        warnings,
                        errors
                    );
                    trait_methods.push(de_insert_function(trait_fn.to_dummy_func(Mode::NonAbi)));
                }

                let type_params_as_type_args = type_parameters
                    .iter()
                    .map(|type_param| TypeArgument {
                        type_id: type_param.type_id,
                        initial_type_id: type_param.initial_type_id,
                        span: type_param.name_ident.span(),
                    })
                    .collect::<Vec<_>>();

                // insert dummy versions of the interfaces for all of the supertraits
                // specifically don't check for conflicting definitions because
                // these are just dummy definitions
                ctx.namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    type_params_as_type_args.clone(),
                    self_type,
                    &trait_methods,
                    &supertrait.name.span(),
                );

                // insert dummy versions of the methods of all of the supertraits
                let dummy_funcs = check!(
                    convert_trait_methods_to_dummy_funcs(ctx.by_ref(), methods),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // specifically don't check for conflicting definitions because
                // these are just dummy definitions
                ctx.namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    type_params_as_type_args,
                    self_type,
                    &dummy_funcs,
                    &supertrait.name.span(),
                );

                // Recurse to insert dummy versions of interfaces and methods of the *super*
                // supertraits
                check!(
                    handle_supertraits(ctx.by_ref(), supertraits),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
            Some(ty::TyDeclaration::AbiDeclaration(_)) => {
                errors.push(CompileError::AbiAsSupertrait {
                    span: supertrait.name.span().clone(),
                })
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: supertrait.name.to_string(),
                span: supertrait.name.span(),
            }),
        }
    }

    ok((), warnings, errors)
}
