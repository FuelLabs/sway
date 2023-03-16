use crate::{
    engine_threading::*, error::*, language::ty, semantic_analysis::*, type_system::priv_prelude::*,
};

use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    cmp::Ordering,
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

impl HashWithEngines for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeParameter {
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

impl EqWithEngines for TypeParameter {}
impl PartialEqWithEngines for TypeParameter {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), engines)
            && self.name_ident == other.name_ident
            && self.trait_constraints.eq(&other.trait_constraints, engines)
    }
}

impl OrdWithEngines for TypeParameter {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let TypeParameter {
            type_id: lti,
            name_ident: ln,
            trait_constraints: ltc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
        } = self;
        let TypeParameter {
            type_id: rti,
            name_ident: rn,
            trait_constraints: rtc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
        } = other;
        ln.cmp(rn)
            .then_with(|| engines.te().get(*lti).cmp(&engines.te().get(*rti), engines))
            .then_with(|| ltc.cmp(rtc, engines))
    }
}

impl SubstTypes for TypeParameter {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_id.subst(type_mapping, engines);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl DebugWithEngines for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:?}",
            self.name_ident,
            engines.help_out(self.type_id)
        )
    }
}

impl fmt::Debug for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name_ident, self.type_id)
    }
}

impl TypeParameter {
    /// Type checks a list of [TypeParameter]s and returns a new list of
    /// [TypeParameter]s and a [SubstList]. This will also insert the list of
    /// [TypeParameter] into the current namespace.
    pub(crate) fn type_check_type_params(
        mut ctx: TypeCheckContext,
        type_params: Vec<TypeParameter>,
        disallow_trait_constraints: bool,
    ) -> CompileResult<(Vec<TypeParameter>, SubstList)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut new_type_params: Vec<TypeParameter> = vec![];
        let mut type_subst_list = ctx
            .namespace
            .type_subst_stack_mut()
            .last()
            .cloned()
            .unwrap_or_default();

        for type_param in type_params.into_iter() {
            if disallow_trait_constraints && !type_param.trait_constraints.is_empty() {
                let errors = vec![CompileError::WhereClauseNotYetSupported {
                    span: type_param.trait_constraints_span,
                }];
                return err(vec![], errors);
            }
            let (subst_list_param, body_type_param) = check!(
                TypeParameter::type_check(ctx.by_ref(), type_param, type_subst_list.len()),
                continue,
                warnings,
                errors
            );
            type_subst_list.push(subst_list_param);
            new_type_params.push(body_type_param);
        }

        if errors.is_empty() {
            ok((new_type_params, type_subst_list), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    /// Type checks a [TypeParameter] (including its [TraitConstraint]s),
    /// inserts it into the current namespace, and returns two new
    /// [TypeParameters]:
    /// 1. A [TypeParameter] to go in the [SubstList].
    /// 2. A [TypeParameter] to go in the body of where this was created.
    ///     References (1) in the [SubstList].
    fn type_check(
        mut ctx: TypeCheckContext,
        type_parameter: TypeParameter,
        next_index: usize,
    ) -> CompileResult<(Self, Self)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

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

        let body_id = type_engine.insert(
            decl_engine,
            TypeInfo::TypeParam {
                index: next_index,
                debug_name: name_ident.clone(),
            },
        );
        let body_type_param = TypeParameter {
            name_ident: name_ident.clone(),
            type_id: body_id,
            initial_type_id,
            trait_constraints: trait_constraints.clone(),
            trait_constraints_span: trait_constraints_span.clone(),
        };

        // Insert the trait constraints into the namespace.
        for trait_constraint in trait_constraints.iter() {
            check!(
                TraitConstraint::insert_into_namespace(ctx.by_ref(), body_id, trait_constraint),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        // Insert the type parameter into the namespace as a dummy type
        // declaration.
        ctx.namespace
            .insert_symbol(
                name_ident.clone(),
                ty::TyDecl::GenericTypeForFunctionScope {
                    name: name_ident.clone(),
                    type_id: body_id,
                },
            )
            .ok(&mut warnings, &mut errors);

        let subst_list_id = type_engine.insert(
            decl_engine,
            TypeInfo::UnknownGeneric {
                name: name_ident.clone(),
                trait_constraints: VecSet(trait_constraints.clone()),
            },
        );
        let subst_list_param = TypeParameter {
            name_ident,
            type_id: subst_list_id,
            initial_type_id,
            trait_constraints,
            trait_constraints_span,
        };

        ok((subst_list_param, body_type_param), warnings, errors)
    }
}
