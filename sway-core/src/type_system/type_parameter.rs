use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct TypeParam {
    pub type_id: TypeId,
    pub(crate) initial_type_id: TypeId,
    pub name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
    pub(crate) trait_constraints_span: Span,
}

impl HashWithEngines for TypeParam {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeParam {
            type_id,
            name_ident,
            trait_constraints,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
        } = self;
        let type_engine = engines.te();
        type_engine.get(*type_id).hash(state, engines);
        name_ident.hash(state);
        trait_constraints.hash(state, engines);
    }
}

impl EqWithEngines for TypeParam {}
impl PartialEqWithEngines for TypeParam {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), engines)
            && self.name_ident == other.name_ident
            && self.trait_constraints.eq(&other.trait_constraints, engines)
    }
}

impl OrdWithEngines for TypeParam {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
        let TypeParam {
            type_id: lti,
            name_ident: ln,
            trait_constraints: ltc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
        } = self;
        let TypeParam {
            type_id: rti,
            name_ident: rn,
            trait_constraints: rtc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
        } = other;
        ln.cmp(rn)
            .then_with(|| {
                type_engine
                    .get(*lti)
                    .cmp(&type_engine.get(*rti), type_engine)
            })
            .then_with(|| ltc.cmp(rtc, type_engine))
    }
}

impl SubstTypes for TypeParam {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_id.subst(type_mapping, engines);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TypeParam {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl Spanned for TypeParam {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl DisplayWithEngines for TypeParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name_ident, engines.help_out(self.type_id))
    }
}

impl fmt::Debug for TypeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name_ident, self.type_id)
    }
}

impl TypeParam {
    /// Type check a list of [TypeParameter] and return a new list of
    /// [TypeParameter]. This will also insert this new list into the current
    /// namespace.
    pub(crate) fn type_check_type_params(
        mut ctx: TypeCheckContext,
        type_params: Vec<TypeParam>,
        disallow_trait_constraints: bool,
    ) -> CompileResult<Vec<TypeParam>> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut new_type_params: Vec<TypeParam> = vec![];

        for type_param in type_params.into_iter() {
            if disallow_trait_constraints && !type_param.trait_constraints.is_empty() {
                let errors = vec![CompileError::WhereClauseNotYetSupported {
                    span: type_param.trait_constraints_span,
                }];
                return err(vec![], errors);
            }
            new_type_params.push(check!(
                TypeParam::type_check(ctx.by_ref(), type_param),
                continue,
                warnings,
                errors
            ));
        }

        if errors.is_empty() {
            ok(new_type_params, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    /// Type checks a [TypeParameter] (including its [TraitConstraint]s) and
    /// inserts into into the current namespace.
    fn type_check(mut ctx: TypeCheckContext, type_parameter: TypeParam) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        let TypeParam {
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

        let type_id = type_engine.insert(
            decl_engine,
            TypeInfo::UnknownGeneric {
                name: name_ident.clone(),
                trait_constraints: VecSet(trait_constraints.clone()),
            },
        );

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

        let type_parameter = TypeParam {
            name_ident,
            type_id,
            initial_type_id,
            trait_constraints,
            trait_constraints_span,
        };
        ok(type_parameter, warnings, errors)
    }

    /// Creates a [DeclMapping] from a list of [TypeParameter]s.
    pub(crate) fn gather_decl_mapping_from_trait_constraints(
        mut ctx: TypeCheckContext,
        type_parameters: &[TypeParam],
        access_span: &Span,
    ) -> CompileResult<DeclMapping> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut original_method_refs: MethodMap = BTreeMap::new();
        let mut impld_method_refs: MethodMap = BTreeMap::new();

        for type_param in type_parameters.iter() {
            let TypeParam {
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
                        ctx.engines()
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

                let (trait_original_method_refs, trait_impld_method_refs) = check!(
                    handle_trait(ctx.by_ref(), *type_id, trait_name, trait_type_arguments),
                    continue,
                    warnings,
                    errors
                );
                original_method_refs.extend(trait_original_method_refs);
                impld_method_refs.extend(trait_impld_method_refs);
            }
        }

        if errors.is_empty() {
            let decl_mapping =
                DeclMapping::from_stub_and_impld_decl_refs(original_method_refs, impld_method_refs);
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
) -> CompileResult<(MethodMap, MethodMap)> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let decl_engine = ctx.decl_engine;

    let mut original_method_refs: MethodMap = BTreeMap::new();
    let mut impld_method_refs: MethodMap = BTreeMap::new();

    match ctx
        .namespace
        .resolve_call_path(trait_name)
        .ok(&mut warnings, &mut errors)
        .cloned()
    {
        Some(ty::TyDeclaration::TraitDeclaration { decl_id, .. }) => {
            let trait_decl = check!(
                CompileResult::from(decl_engine.get_trait(&decl_id, &trait_name.suffix.span())),
                return err(warnings, errors),
                warnings,
                errors
            );

            let (trait_original_method_refs, trait_method_refs, trait_impld_method_refs) = check!(
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
            original_method_refs.extend(trait_original_method_refs);
            original_method_refs.extend(trait_method_refs);
            impld_method_refs.extend(trait_impld_method_refs);

            for supertrait in trait_decl.supertraits.iter() {
                let (supertrait_original_method_refs, supertrait_impld_method_refs) = check!(
                    handle_trait(ctx.by_ref(), type_id, &supertrait.name, &[]),
                    continue,
                    warnings,
                    errors
                );
                original_method_refs.extend(supertrait_original_method_refs);
                impld_method_refs.extend(supertrait_impld_method_refs);
            }
        }
        _ => errors.push(CompileError::TraitNotFound {
            name: trait_name.to_string(),
            span: trait_name.span(),
        }),
    }

    if errors.is_empty() {
        ok((original_method_refs, impld_method_refs), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
