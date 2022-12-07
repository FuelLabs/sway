use crate::{
    declaration_engine::*,
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
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
impl HashWithEngines for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        type_engine
            .look_up_type_id(self.type_id)
            .hash(state, type_engine);
        self.name_ident.hash(state);
        self.trait_constraints.hash(state, type_engine);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TypeParameter {}
impl PartialEqWithEngines for TypeParameter {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        type_engine
            .look_up_type_id(self.type_id)
            .eq(&type_engine.look_up_type_id(other.type_id), engines)
            && self.name_ident == other.name_ident
            && self.trait_constraints.eq(&other.trait_constraints, engines)
    }
}

impl CopyTypes for TypeParameter {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.type_id.copy_types(type_mapping, engines);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
    }
}

impl ReplaceSelfType for TypeParameter {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl DisplayWithEngines for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name_ident, engines.help_out(self.type_id))
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

        let type_id = ctx.type_engine.insert_type(TypeInfo::UnknownGeneric {
            name: name_ident.clone(),
            trait_constraints: VecSet(trait_constraints.clone()),
        });

        // Insert the trait constraints into the namespace.
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
    /// append the current TypeParameter as a `fuels_types::TypeDeclaration`.
    pub(crate) fn get_json_type_parameter(
        &self,
        type_engine: &TypeEngine,
        types: &mut Vec<fuels_types::TypeDeclaration>,
    ) -> usize {
        let type_parameter = fuels_types::TypeDeclaration {
            type_id: self.initial_type_id.index(),
            type_field: self
                .initial_type_id
                .get_json_type_str(type_engine, self.type_id),
            components: self.initial_type_id.get_json_type_components(
                type_engine,
                types,
                self.type_id,
            ),
            type_parameters: None,
        };
        types.push(type_parameter);
        self.initial_type_id.index()
    }

    /// Creates a [DeclMapping] from a list of [TypeParameter]s.
    pub(crate) fn gather_decl_mapping_from_trait_constraints(
        mut ctx: TypeCheckContext,
        type_parameters: &[TypeParameter],
        access_span: &Span,
    ) -> CompileResult<DeclMapping> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut original_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
        let mut impld_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

        for type_param in type_parameters.iter() {
            let TypeParameter {
                type_id,
                trait_constraints,
                ..
            } = type_param;

            // Check to see if the trait constraints are satisfied.
            check!(
                ctx.namespace
                    .implemented_traits
                    .check_if_trait_constraints_are_satisfied_for_type(
                        *type_id,
                        trait_constraints,
                        access_span,
                        ctx.type_engine,
                    ),
                continue,
                warnings,
                errors
            );

            for trait_constraint in trait_constraints.iter() {
                let TraitConstraint {
                    trait_name,
                    type_arguments: trait_type_arguments,
                } = trait_constraint;

                let (trait_original_method_ids, trait_impld_method_ids) = check!(
                    handle_trait(ctx.by_ref(), *type_id, trait_name, trait_type_arguments),
                    continue,
                    warnings,
                    errors
                );
                original_method_ids.extend(trait_original_method_ids);
                impld_method_ids.extend(trait_impld_method_ids);
            }
        }

        if errors.is_empty() {
            let decl_mapping =
                DeclMapping::from_original_and_new_decl_ids(original_method_ids, impld_method_ids);
            ok(decl_mapping, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

fn handle_trait(
    mut ctx: TypeCheckContext,
    type_id: TypeId,
    trait_name: &CallPath,
    type_arguments: &[TypeArgument],
) -> CompileResult<(
    BTreeMap<Ident, DeclarationId>,
    BTreeMap<Ident, DeclarationId>,
)> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let declaration_engine = ctx.declaration_engine;

    let mut original_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
    let mut impld_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

    match ctx
        .namespace
        .resolve_call_path(trait_name)
        .ok(&mut warnings, &mut errors)
        .cloned()
    {
        Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
            let trait_decl = check!(
                CompileResult::from(declaration_engine.get_trait(decl_id, &trait_name.span())),
                return err(warnings, errors),
                warnings,
                errors
            );

            let (trait_original_method_ids, trait_method_ids, trait_impld_method_ids) = check!(
                trait_decl.retrieve_interface_surface_and_methods_and_implemented_methods_for_type(
                    ctx.by_ref(),
                    type_id,
                    trait_name,
                    type_arguments
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            original_method_ids.extend(trait_original_method_ids);
            original_method_ids.extend(trait_method_ids);
            impld_method_ids.extend(trait_impld_method_ids);

            for supertrait in trait_decl.supertraits.iter() {
                let (supertrait_original_method_ids, supertrait_impld_method_ids) = check!(
                    handle_trait(ctx.by_ref(), type_id, &supertrait.name, &[]),
                    continue,
                    warnings,
                    errors
                );
                original_method_ids.extend(supertrait_original_method_ids);
                impld_method_ids.extend(supertrait_impld_method_ids);
            }
        }
        _ => errors.push(CompileError::TraitNotFound {
            name: trait_name.to_string(),
            span: trait_name.span(),
        }),
    }

    if errors.is_empty() {
        ok((original_method_ids, impld_method_ids), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
