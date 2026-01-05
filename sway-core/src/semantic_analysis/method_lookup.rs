use std::collections::{BTreeMap, HashSet};

use itertools::Itertools;

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Ident, Spanned};

use crate::{
    decl_engine::DeclRefFunction,
    language::{
        parsed::MethodName,
        ty::{self, TyFunctionDisplay},
        CallPath, QualifiedCallPath,
    },
    namespace::{
        IsImplInterfaceSurface, Module, ModulePath, ResolvedTraitImplItem, TraitKey, TraitMap,
        TraitSuffix,
    },
    type_system::{GenericArgument, TypeId, TypeInfo, TypeParameter},
    EnforceTypeArguments, Engines, TraitConstraint, UnifyCheck,
};

use super::{
    type_check_context::TypeCheckContext,
    type_resolve::{resolve_type, VisibilityCheck},
};

struct MethodCandidate {
    decl_ref: DeclRefFunction,
    params: Vec<TypeId>,
    ret: TypeId,
    is_contract_call: bool,
}

enum MatchScore {
    Exact,
    Coercible,
    Incompatible,
}

#[derive(Clone)]
pub(crate) struct CandidateTraitItem {
    item: ty::TyTraitItem,
    trait_key: TraitKey,
    original_type_id: TypeId,
    resolved_type_id: TypeId,
}

fn trait_paths_equivalent(
    allowed: &CallPath<TraitSuffix>,
    other: &CallPath<TraitSuffix>,
    unify_check: &UnifyCheck,
) -> bool {
    if allowed
        .prefixes
        .iter()
        .zip(other.prefixes.iter())
        .any(|(a, b)| a != b)
    {
        return false;
    }
    if allowed.suffix.name != other.suffix.name {
        return false;
    }
    if allowed.suffix.args.len() != other.suffix.args.len() {
        return false;
    }
    allowed
        .suffix
        .args
        .iter()
        .zip(other.suffix.args.iter())
        .all(|(a, b)| unify_check.check(a.type_id(), b.type_id()))
}

type TraitImplId = crate::decl_engine::DeclId<ty::TyImplSelfOrTrait>;
type GroupingKey = (TraitImplId, Option<TypeId>);

struct GroupingResult {
    trait_methods: BTreeMap<GroupingKey, DeclRefFunction>,
    impl_self_method: Option<DeclRefFunction>,
    qualified_call_path: Option<QualifiedCallPath>,
}

impl TypeCheckContext<'_> {
    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find items matching in the namespace.
    pub(crate) fn find_items_for_type(
        &self,
        handler: &Handler,
        type_id: TypeId,
        item_prefix: &ModulePath,
        item_name: &Ident,
        method_name: &Option<&MethodName>,
    ) -> Result<Vec<CandidateTraitItem>, ErrorEmitted> {
        let type_engine = self.engines.te();
        let original_type_id = type_id;

        // If the type that we are looking for is the error recovery type, then
        // we want to return the error case without creating a new error
        // message.
        if let TypeInfo::ErrorRecovery(err) = &*type_engine.get(type_id) {
            return Err(*err);
        }

        // resolve the type
        let resolved_type_id = resolve_type(
            handler,
            self.engines(),
            self.namespace(),
            item_prefix,
            type_id,
            &item_name.span(),
            EnforceTypeArguments::No,
            None,
            self.self_type(),
            &self.subst_ctx(handler),
            VisibilityCheck::Yes,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        // grab the local module
        let local_module = self
            .namespace()
            .require_module_from_absolute_path(handler, &self.namespace().current_mod_path)?;

        // grab the local items from the local module
        let mut matching_items = vec![];
        let mut filter_item = |item: ResolvedTraitImplItem, trait_key: TraitKey| match &item {
            ResolvedTraitImplItem::Parsed(_) => todo!(),
            ResolvedTraitImplItem::Typed(ty_item) => match ty_item {
                ty::TyTraitItem::Fn(decl_ref) if decl_ref.name() == item_name => {
                    matching_items.push(CandidateTraitItem {
                        item: ty_item.clone(),
                        trait_key: trait_key.clone(),
                        original_type_id,
                        resolved_type_id,
                    });
                }
                ty::TyTraitItem::Constant(decl_ref) if decl_ref.name() == item_name => {
                    matching_items.push(CandidateTraitItem {
                        item: ty_item.clone(),
                        trait_key: trait_key.clone(),
                        original_type_id,
                        resolved_type_id,
                    });
                }
                ty::TyTraitItem::Type(decl_ref) if decl_ref.name() == item_name => {
                    matching_items.push(CandidateTraitItem {
                        item: ty_item.clone(),
                        trait_key: trait_key.clone(),
                        original_type_id,
                        resolved_type_id,
                    });
                }
                _ => {}
            },
        };

        TraitMap::find_items_and_trait_key_for_type(
            local_module,
            self.engines,
            resolved_type_id,
            &mut filter_item,
        );

        // grab the items from where the argument type is declared
        if let Some(MethodName::FromTrait { .. }) = method_name {
            let type_module = self.get_namespace_module_from_type_id(resolved_type_id);
            if let Ok(type_module) = type_module {
                TraitMap::find_items_and_trait_key_for_type(
                    type_module,
                    self.engines,
                    resolved_type_id,
                    &mut filter_item,
                );
            }
        }

        if item_prefix != self.namespace().current_mod_path.as_slice() {
            // grab the module where the type itself is declared
            let type_module = self
                .namespace()
                .require_module_from_absolute_path(handler, item_prefix)?;

            // grab the items from where the type is declared
            TraitMap::find_items_and_trait_key_for_type(
                type_module,
                self.engines,
                resolved_type_id,
                &mut filter_item,
            );
        }

        Ok(matching_items)
    }

    fn get_namespace_module_from_type_id(&self, type_id: TypeId) -> Result<&Module, ErrorEmitted> {
        let type_info = self.engines().te().get(type_id);
        if type_info.is_alias() {
            if let TypeInfo::Alias { ty, .. } = &*type_info {
                return self.get_namespace_module_from_type_id(ty.type_id);
            }
        }

        let handler = Handler::default();
        let call_path = match *type_info {
            TypeInfo::Enum(decl_id) => self.engines().de().get_enum(&decl_id).call_path.clone(),
            TypeInfo::Struct(decl_id) => self.engines().de().get_struct(&decl_id).call_path.clone(),
            _ => {
                return Err(handler.emit_err(CompileError::Internal(
                    "No call path for type id",
                    Span::dummy(),
                )))
            }
        };

        let call_path = call_path.rshift();
        self.namespace()
            .require_module_from_absolute_path(&handler, &call_path.as_vec_ident())
    }

    fn default_numeric_if_needed(
        &self,
        handler: &Handler,
        type_id: TypeId,
        method_name: &Ident,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = self.engines.te();

        // Default numeric types to u64
        if type_engine.contains_numeric(self.engines, type_id) {
            // While collecting unifications we don't decay numeric and will ignore this error.
            if self.collecting_unifications() {
                return Err(handler.emit_err(CompileError::MethodNotFound {
                    called_method: method_name.into(),
                    expected_signature: method_name.clone().as_str().to_string(),
                    type_name: self.engines.help_out(type_id).to_string(),
                    matching_methods: vec![],
                }));
            }
            type_engine.decay_numeric(handler, self.engines, type_id, &method_name.span())?;
        }

        Ok(())
    }

    /// Collect all candidate trait items that might provide the method:
    /// - Items directly available for `type_id`
    /// - Plus items from any annotation-type inner that can coerce to `type_id`
    fn collect_candidate_items(
        &self,
        handler: &Handler,
        type_id: TypeId,
        method_prefix: &ModulePath,
        method_ident: &Ident,
        method_name: &Option<&MethodName>,
    ) -> Result<Vec<ty::TyTraitItem>, ErrorEmitted> {
        // Start with items for the concrete type.
        let mut items =
            self.find_items_for_type(handler, type_id, method_prefix, method_ident, method_name)?;

        if method_name.is_none() {
            return Ok(items.into_iter().map(|candidate| candidate.item).collect());
        }

        if let Some(method_name) = method_name {
            let method_constraints = self.trait_constraints_from_method_name(handler, method_name);
            self.filter_items_by_trait_access(&mut items, method_constraints.as_ref());
        }

        Ok(items.into_iter().map(|candidate| candidate.item).collect())
    }

    fn trait_constraints_from_method_name(
        &self,
        handler: &Handler,
        method_name: &MethodName,
    ) -> Option<(TypeId, Vec<TraitConstraint>)> {
        let _ = handler;
        let MethodName::FromType {
            call_path_binding, ..
        } = method_name
        else {
            return None;
        };

        let (_, type_ident) = &call_path_binding.inner.suffix;
        let resolve_handler = Handler::default();
        let Ok((resolved_decl, _)) = self.namespace().current_module().resolve_symbol(
            &resolve_handler,
            self.engines(),
            type_ident,
        ) else {
            return None;
        };

        match resolved_decl.expect_typed() {
            ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                type_id,
                ..
            }) => {
                let mut constraints = Vec::new();
                let mut visited = HashSet::new();
                self.collect_trait_constraints_recursive(type_id, &mut constraints, &mut visited);
                Some((type_id, constraints))
            }
            _ => None,
        }
    }

    /// Filter the candidate trait items so that only methods whose traits satisfy the relevant
    /// trait bounds remain. Groups corresponding to other generic parameters are left untouched.
    fn filter_items_by_trait_access(
        &self,
        items: &mut Vec<CandidateTraitItem>,
        method_constraints: Option<&(TypeId, Vec<TraitConstraint>)>,
    ) {
        if items.is_empty() {
            return;
        }

        // Group candidates by the (possibly still generic) type they originated from so we can
        // later apply trait bounds per generic parameter.
        let mut grouped: BTreeMap<TypeId, Vec<CandidateTraitItem>> = BTreeMap::new();
        for item in items.drain(..) {
            grouped
                .entry(
                    self.engines
                        .te()
                        .get_unaliased_type_id(item.resolved_type_id),
                )
                .or_default()
                .push(item);
        }

        // If this lookup is resolving a concrete method name, pre-compute the generic type whose
        // bounds we collected so we only apply those bounds to matching groups.
        let method_constraint_info = method_constraints.and_then(|(type_id, constraints)| {
            (!constraints.is_empty()).then_some((*type_id, constraints.as_slice()))
        });

        let type_engine = self.engines.te();
        let mut filtered = Vec::new();

        for (type_id, group) in grouped {
            let type_info = type_engine.get(type_id);
            if !matches!(
                *type_info,
                TypeInfo::UnknownGeneric { .. } | TypeInfo::Placeholder(_)
            ) {
                filtered.extend(group);
                continue;
            }

            let (interface_items, impl_items): (Vec<_>, Vec<_>) =
                group.into_iter().partition(|item| {
                    matches!(
                        item.trait_key.is_impl_interface_surface,
                        IsImplInterfaceSurface::Yes
                    )
                });

            // Only groups born from the same generic parameter as the method call need to honour
            // the method's trait bounds. Other generic parameters can pass through untouched.
            let extra_constraints =
                method_constraint_info.and_then(|(constraint_type_id, constraints)| {
                    let applies_to_group =
                        interface_items.iter().chain(impl_items.iter()).any(|item| {
                            self.engines
                                .te()
                                .get_unaliased_type_id(item.original_type_id)
                                == constraint_type_id
                        });

                    applies_to_group.then_some(constraints)
                });

            let allowed_traits =
                self.allowed_traits_for_type(type_id, &interface_items, extra_constraints);

            if allowed_traits.is_empty() {
                filtered.extend(interface_items);
                filtered.extend(impl_items);
                continue;
            }

            if !impl_items.is_empty() {
                let mut retained_impls = Vec::new();
                for item in impl_items {
                    if self.trait_key_matches_allowed(&item.trait_key, &allowed_traits) {
                        retained_impls.push(item);
                    }
                }

                if !retained_impls.is_empty() {
                    filtered.extend(retained_impls);
                    filtered.extend(interface_items);
                    continue;
                }
            }

            // No impl methods matched the bounds, so fall back to the interface placeholders.
            filtered.extend(interface_items);
        }

        *items = filtered;
    }

    /// Build the list of trait paths that should remain visible for the given `type_id` when
    /// resolving a method. This includes traits that supplied the interface surface entries as well
    /// as any traits required by bounds on the generic parameter.
    fn allowed_traits_for_type(
        &self,
        type_id: TypeId,
        interface_items: &[CandidateTraitItem],
        extra_constraints: Option<&[TraitConstraint]>,
    ) -> Vec<CallPath<TraitSuffix>> {
        // Seed the allow-list with the traits that provided the interface items. They act as
        // fallbacks whenever no concrete implementation matches the bounds.
        let mut allowed: Vec<CallPath<TraitSuffix>> = interface_items
            .iter()
            .map(|item| item.trait_key.name.as_ref().clone())
            .collect();

        // Add trait bounds declared on the type parameter itself (recursively following inherited
        // bounds) so they can participate in disambiguation.
        let mut constraints = Vec::new();
        let mut visited = HashSet::new();
        self.collect_trait_constraints_recursive(type_id, &mut constraints, &mut visited);

        for constraint in constraints {
            let canonical = constraint
                .trait_name
                .to_canonical_path(self.engines(), self.namespace());
            allowed.push(CallPath {
                prefixes: canonical.prefixes,
                suffix: TraitSuffix {
                    name: canonical.suffix,
                    args: constraint.type_arguments.clone(),
                },
                callpath_type: canonical.callpath_type,
            });
        }

        // Method-specific bounds (for example from `fn foo<T: Trait>()`) are supplied separately,
        // include them so only the permitted traits remain candidates after filtering.
        if let Some(extra) = extra_constraints {
            for constraint in extra {
                let canonical = constraint
                    .trait_name
                    .to_canonical_path(self.engines(), self.namespace());
                allowed.push(CallPath {
                    prefixes: canonical.prefixes,
                    suffix: TraitSuffix {
                        name: canonical.suffix,
                        args: constraint.type_arguments.clone(),
                    },
                    callpath_type: canonical.callpath_type,
                });
            }
        }

        self.dedup_allowed_traits(allowed)
    }

    /// Remove equivalent trait paths (up to type-argument unification) from the allow-list to
    /// avoid redundant comparisons later on.
    fn dedup_allowed_traits(
        &self,
        allowed: Vec<CallPath<TraitSuffix>>,
    ) -> Vec<CallPath<TraitSuffix>> {
        let mut deduped = Vec::new();
        let unify_check = UnifyCheck::constraint_subset(self.engines);

        for entry in allowed.into_iter() {
            if deduped
                .iter()
                .any(|existing| trait_paths_equivalent(existing, &entry, &unify_check))
            {
                continue;
            }
            deduped.push(entry);
        }

        deduped
    }

    /// Recursively collect trait constraints that apply to `type_id`, following aliases,
    /// placeholders, and chains of generic parameters.
    fn collect_trait_constraints_recursive(
        &self,
        type_id: TypeId,
        acc: &mut Vec<TraitConstraint>,
        visited: &mut HashSet<TypeId>,
    ) {
        let type_engine = self.engines.te();
        let type_id = type_engine.get_unaliased_type_id(type_id);
        if !visited.insert(type_id) {
            return;
        }

        match &*type_engine.get(type_id) {
            TypeInfo::UnknownGeneric {
                trait_constraints,
                parent,
                ..
            } => {
                acc.extend(trait_constraints.iter().cloned());
                if let Some(parent_id) = parent {
                    self.collect_trait_constraints_recursive(*parent_id, acc, visited);
                }
            }
            TypeInfo::Placeholder(TypeParameter::Type(generic)) => {
                acc.extend(generic.trait_constraints.iter().cloned());
                self.collect_trait_constraints_recursive(generic.type_id, acc, visited);
            }
            TypeInfo::Alias { ty, .. } => {
                self.collect_trait_constraints_recursive(ty.type_id, acc, visited);
            }
            _ => {}
        }
    }

    /// Keep only the trait methods whose originating trait satisfies the collected constraints.
    fn retain_trait_methods_matching_constraints(
        &self,
        trait_methods: &mut BTreeMap<GroupingKey, DeclRefFunction>,
        constraints: &[TraitConstraint],
    ) {
        if constraints.is_empty() {
            return;
        }

        // Precompute the canonical trait paths allowed by the constraints so we can filter impls.
        let allowed_traits = constraints
            .iter()
            .map(|constraint| {
                let canonical = constraint
                    .trait_name
                    .to_canonical_path(self.engines(), self.namespace());
                CallPath {
                    prefixes: canonical.prefixes,
                    suffix: TraitSuffix {
                        name: canonical.suffix,
                        args: constraint.type_arguments.clone(),
                    },
                    callpath_type: canonical.callpath_type,
                }
            })
            .collect::<Vec<_>>();

        if allowed_traits.is_empty() {
            return;
        }

        let mut filtered = trait_methods.clone();
        let unify_check = UnifyCheck::constraint_subset(self.engines);

        filtered.retain(|(_impl_id, _), decl_ref| {
            let method = self.engines.de().get_function(decl_ref);
            let Some(ty::TyDecl::ImplSelfOrTrait(impl_ref)) = method.implementing_type.as_ref()
            else {
                return true;
            };

            let impl_decl = self.engines.de().get_impl_self_or_trait(&impl_ref.decl_id);

            // Inherent impls have no trait declaration, keep them untouched.
            if impl_decl.trait_decl_ref.is_none() {
                return true;
            }

            // Build the canonical trait path and check whether it matches any of the trait bounds
            // collected for this lookup. Only methods provided by traits that satisfy the bounds
            // remain candidates for disambiguation.
            let canonical = impl_decl
                .trait_name
                .to_canonical_path(self.engines(), self.namespace());
            let candidate = CallPath {
                prefixes: canonical.prefixes,
                suffix: TraitSuffix {
                    name: canonical.suffix,
                    args: impl_decl.trait_type_arguments.clone(),
                },
                callpath_type: canonical.callpath_type,
            };

            allowed_traits
                .iter()
                .any(|allowed| trait_paths_equivalent(allowed, &candidate, &unify_check))
        });

        if !filtered.is_empty() {
            *trait_methods = filtered;
        }
    }

    fn trait_key_matches_allowed(
        &self,
        trait_key: &TraitKey,
        allowed_traits: &[CallPath<TraitSuffix>],
    ) -> bool {
        if allowed_traits.is_empty() {
            return false;
        }
        let call_path = trait_key.name.as_ref();
        let unify_check = UnifyCheck::constraint_subset(self.engines);

        allowed_traits
            .iter()
            .any(|allowed| trait_paths_equivalent(allowed, call_path, &unify_check))
    }

    /// Convert collected items to just the method decl refs we care about.
    fn items_to_method_refs(&self, items: Vec<ty::TyTraitItem>) -> Vec<DeclRefFunction> {
        items
            .into_iter()
            .filter_map(|item| match item {
                ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                _ => None,
            })
            .collect()
    }

    fn to_method_candidate(&self, decl_ref: &DeclRefFunction) -> MethodCandidate {
        let decl_engine = self.engines.de();
        let fn_decl = decl_engine.get_function(decl_ref);
        MethodCandidate {
            decl_ref: decl_ref.clone(),
            params: fn_decl
                .parameters
                .iter()
                .map(|p| p.type_argument.type_id)
                .collect(),
            ret: fn_decl.return_type.type_id,
            is_contract_call: fn_decl.is_contract_call,
        }
    }

    /// Decide whether `cand` matches the given argument and annotation types.
    fn score_method_candidate(
        &self,
        cand: &MethodCandidate,
        argument_types: &[TypeId],
        annotation_type: TypeId,
    ) -> MatchScore {
        let eq_check = UnifyCheck::constraint_subset(self.engines).with_unify_ref_mut(false);
        let coercion_check = UnifyCheck::coercion(self.engines).with_ignore_generic_names(true);

        // Handle "self" for contract calls.
        let args_len_diff = if cand.is_contract_call && !argument_types.is_empty() {
            1
        } else {
            0
        };

        // Parameter count must match.
        if cand.params.len() != argument_types.len().saturating_sub(args_len_diff) {
            return MatchScore::Incompatible;
        }

        // Param-by-param check.
        let mut all_exact = true;
        for (p, a) in cand
            .params
            .iter()
            .zip(argument_types.iter().skip(args_len_diff))
        {
            if eq_check.check(*a, *p) {
                continue;
            }
            if coercion_check.check(*a, *p) {
                all_exact = false;
                continue;
            }
            return MatchScore::Incompatible;
        }

        let type_engine = self.engines.te();
        let ann = &*type_engine.get(annotation_type);
        let ret_ok = matches!(ann, TypeInfo::Unknown)
            || matches!(&*type_engine.get(cand.ret), TypeInfo::Never)
            || coercion_check.check(annotation_type, cand.ret);

        if !ret_ok {
            return MatchScore::Incompatible;
        }

        if all_exact {
            MatchScore::Exact
        } else {
            MatchScore::Coercible
        }
    }

    /// Keep only compatible candidates (coercible or exact).
    fn filter_method_candidates_by_signature(
        &self,
        decl_refs: &Vec<DeclRefFunction>,
        argument_types: &[TypeId],
        annotation_type: TypeId,
    ) -> Vec<MethodCandidate> {
        let mut out = Vec::new();
        for r in decl_refs {
            let cand = self.to_method_candidate(r);
            match self.score_method_candidate(&cand, argument_types, annotation_type) {
                MatchScore::Exact | MatchScore::Coercible => out.push(cand),
                MatchScore::Incompatible => {}
            }
        }
        out
    }

    /// Group signature-compatible method decl refs by their originating impl block,
    /// optionally filtering by a qualified trait path.
    fn group_by_trait_impl(
        &self,
        handler: &Handler,
        method_name: &Option<&MethodName>,
        method_decl_refs: &[DeclRefFunction],
    ) -> Result<GroupingResult, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let type_engine = self.engines.te();
        let eq_check = UnifyCheck::constraint_subset(self.engines);

        // Extract `<... as Trait::<Args>>::method` info, if present.
        let (qualified_call_path, trait_method_name_binding_type_args): (
            Option<QualifiedCallPath>,
            Option<Vec<_>>,
        ) = match method_name {
            Some(MethodName::FromQualifiedPathRoot { as_trait, .. }) => {
                match &*type_engine.get(*as_trait) {
                    TypeInfo::Custom {
                        qualified_call_path: cp,
                        type_arguments,
                    } => (Some(cp.clone()), type_arguments.clone()),
                    _ => (None, None),
                }
            }
            _ => (None, None),
        };

        // Helper: compare two type arguments after resolution.
        let types_equal = |a: (&GenericArgument, &GenericArgument)| -> Result<bool, ErrorEmitted> {
            let (p1, p2) = a;
            let p1_id = self.resolve_type(
                handler,
                p1.type_id(),
                &p1.span(),
                EnforceTypeArguments::Yes,
                None,
            )?;
            let p2_id = self.resolve_type(
                handler,
                p2.type_id(),
                &p2.span(),
                EnforceTypeArguments::Yes,
                None,
            )?;
            Ok(eq_check.check(p1_id, p2_id))
        };

        // Helper: check whether this impl matches the optional qualified trait filter.
        let matches_trait_filter =
            |trait_decl: &ty::TyImplSelfOrTrait| -> Result<bool, ErrorEmitted> {
                // If there's no qualified trait filter, accept everything.
                let Some(qcp) = &qualified_call_path else {
                    return Ok(true);
                };

                // Trait name must match the one from the qualified path.
                if trait_decl.trait_name != qcp.clone().to_call_path(handler)? {
                    return Ok(false);
                }

                // If the qualified path provided type arguments, they must match the impl's.
                if let Some(params) = &trait_method_name_binding_type_args {
                    if params.len() != trait_decl.trait_type_arguments.len() {
                        return Ok(false);
                    }
                    for pair in params.iter().zip(trait_decl.trait_type_arguments.iter()) {
                        if !types_equal(pair)? {
                            return Ok(false);
                        }
                    }
                }
                Ok(true)
            };

        let mut trait_methods: BTreeMap<GroupingKey, DeclRefFunction> = BTreeMap::new();
        let mut impl_self_method: Option<DeclRefFunction> = None;

        for method_ref in method_decl_refs {
            let method = decl_engine.get_function(method_ref);

            // Only keep methods from an impl block (trait or inherent).
            let Some(ty::TyDecl::ImplSelfOrTrait(impl_trait)) = method.implementing_type.as_ref()
            else {
                continue;
            };

            let trait_decl = decl_engine.get_impl_self_or_trait(&impl_trait.decl_id);
            if !matches_trait_filter(&trait_decl)? {
                continue;
            }

            let key: GroupingKey = (impl_trait.decl_id, method.implementing_for);

            // Prefer the method that is type-check finalized when conflicting.
            match trait_methods.get_mut(&key) {
                Some(existing_ref) => {
                    let existing = decl_engine.get_function(existing_ref);
                    if !existing.is_type_check_finalized || method.is_type_check_finalized {
                        *existing_ref = method_ref.clone();
                    }
                }
                None => {
                    trait_methods.insert(key, method_ref.clone());
                }
            }

            // Track presence of an inherent impl so we can prefer it later.
            if trait_decl.trait_decl_ref.is_none() {
                impl_self_method = Some(method_ref.clone());
            }
        }

        Ok(GroupingResult {
            trait_methods,
            impl_self_method,
            qualified_call_path,
        })
    }

    fn prefer_non_blanket_impls(&self, trait_methods: &mut BTreeMap<GroupingKey, DeclRefFunction>) {
        let decl_engine = self.engines.de();

        let non_blanket_impl_exists = {
            trait_methods.values().any(|v| {
                let m = decl_engine.get_function(v);
                !m.is_from_blanket_impl(self.engines)
            })
        };

        if non_blanket_impl_exists {
            trait_methods.retain(|_, v| {
                let m = decl_engine.get_function(v);
                !m.is_from_blanket_impl(self.engines)
            });
        }
    }

    /// Format the trait name (including type arguments) for diagnostic output.
    fn trait_sig_string(&self, impl_id: &TraitImplId) -> String {
        let de = self.engines.de();
        let trait_decl = de.get_impl_self_or_trait(impl_id);
        if trait_decl.trait_type_arguments.is_empty() {
            trait_decl.trait_name.suffix.to_string()
        } else {
            let args = trait_decl
                .trait_type_arguments
                .iter()
                .map(|ga| self.engines.help_out(ga).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", trait_decl.trait_name.suffix, args)
        }
    }

    /// Choose the final method candidate from the grouped impls, handling inherent impl
    /// precedence, exact matches, and ambiguity diagnostics.
    fn select_method_from_grouped(
        &self,
        handler: &Handler,
        method_name: &Ident,
        type_id: TypeId,
        trait_methods: &BTreeMap<GroupingKey, DeclRefFunction>,
        impl_self_method: &Option<DeclRefFunction>,
        call_site_type_id: Option<TypeId>,
    ) -> Result<Option<DeclRefFunction>, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let eq_check = UnifyCheck::constraint_subset(self.engines);

        match trait_methods.len() {
            0 => Ok(None),
            1 => Ok(trait_methods.values().next().cloned()),
            _ => {
                // Prefer inherent impl when mixed with trait methods.
                if let Some(impl_self) = impl_self_method {
                    return Ok(Some(impl_self.clone()));
                }

                // Exact implementing type wins.
                let mut exact = vec![];
                for r in trait_methods.values() {
                    let m = decl_engine.get_function(r);
                    if let Some(impl_for) = m.implementing_for {
                        if eq_check.with_unify_ref_mut(false).check(impl_for, type_id) {
                            exact.push(r.clone());
                        }
                    }
                }
                if exact.len() == 1 {
                    return Ok(Some(exact.remove(0)));
                }

                // Ambiguity: rebuild strings from impl ids.
                let call_site_type_name =
                    call_site_type_id.map(|id| self.engines.help_out(id).to_string());
                let fallback_type_name = self.engines.help_out(type_id).to_string();

                let mut trait_strings = trait_methods
                    .keys()
                    .map(|(impl_id, implementing_for)| {
                        let trait_str = self.trait_sig_string(impl_id);
                        let impl_for_str = if let Some(name) = call_site_type_name.as_ref() {
                            name.clone()
                        } else if let Some(t) = implementing_for {
                            self.engines.help_out(t).to_string()
                        } else {
                            fallback_type_name.clone()
                        };
                        (trait_str, impl_for_str)
                    })
                    .collect::<Vec<(String, String)>>();
                trait_strings.sort();

                Err(
                    handler.emit_err(CompileError::MultipleApplicableItemsInScope {
                        item_name: method_name.as_str().to_string(),
                        item_kind: "function".to_string(),
                        as_traits: trait_strings,
                        span: method_name.span(),
                    }),
                )
            }
        }
    }

    /// Produce human-readable strings for potential candidates to show in diagnostics.
    fn format_candidate_summaries_for_error<'a>(
        engines: &'a Engines,
        decl_refs: &'a [DeclRefFunction],
    ) -> impl Iterator<Item = String> + 'a {
        let de = engines.de();
        let fn_display = TyFunctionDisplay::full().without_self_param_type();

        decl_refs.iter().map(move |r| {
            let m = de.get_function(r);
            fn_display.display(&m, engines)
        })
    }

    /// Given a `method_name` and a `type_id`, find that method on that type in the namespace.
    ///
    /// `annotation_type` is the expected method return type.
    ///
    /// Requires `argument_types` because:
    /// - standard operations like +, <=, etc. are called like "std::ops::<operation>" and the
    ///   actual self type of the trait implementation is determined by the passed argument type.
    /// - we can have several implementations of generic traits for different types, that can
    ///   result in a method of a same name, but with different type arguments.
    ///
    /// This function will emit a [CompileError::MethodNotFound] if the method is not found.
    ///
    /// Note that _method_ here means **any function associated to a type**, with or without
    /// the `self` argument.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn find_method_for_type(
        &self,
        handler: &Handler,
        type_id: TypeId,
        method_prefix: &ModulePath,
        method_ident: &Ident,
        annotation_type: TypeId,
        arguments_types: &[TypeId],
        method_name: Option<&MethodName>,
    ) -> Result<DeclRefFunction, ErrorEmitted> {
        let type_engine = self.engines.te();

        self.default_numeric_if_needed(handler, type_id, method_ident)?;

        let matching_items = self.collect_candidate_items(
            handler,
            type_id,
            method_prefix,
            method_ident,
            &method_name,
        )?;

        let matching_method_decl_refs = self.items_to_method_refs(matching_items);

        let candidates = self.filter_method_candidates_by_signature(
            &matching_method_decl_refs,
            arguments_types,
            annotation_type,
        );

        let mut matching_methods = HashSet::<String>::new();

        let mut qualified_call_path: Option<QualifiedCallPath> = None;

        if !candidates.is_empty() {
            let maybe_method_decl_refs: Vec<DeclRefFunction> =
                candidates.iter().map(|c| c.decl_ref.clone()).collect();

            let GroupingResult {
                mut trait_methods,
                impl_self_method,
                qualified_call_path: qcp,
            } = self.group_by_trait_impl(handler, &method_name, &maybe_method_decl_refs)?;
            qualified_call_path = qcp;

            let method_constraints =
                method_name.and_then(|name| self.trait_constraints_from_method_name(handler, name));

            if let Some((_, constraints)) = method_constraints.as_ref() {
                self.retain_trait_methods_matching_constraints(&mut trait_methods, constraints);
            }

            // Prefer non-blanket impls when any concrete impl exists.
            self.prefer_non_blanket_impls(&mut trait_methods);

            // Final selection / ambiguity handling.
            if let Some(pick) = self.select_method_from_grouped(
                handler,
                method_ident,
                type_id,
                &trait_methods,
                &impl_self_method,
                method_constraints.as_ref().map(|(type_id, _)| *type_id),
            )? {
                return Ok(pick.get_method_safe_to_unify(handler, self.engines, type_id));
            }

            if qualified_call_path.is_none() {
                if let Some(first) = maybe_method_decl_refs.first() {
                    return Ok(first.get_method_safe_to_unify(handler, self.engines, type_id));
                }
            }
        } else {
            // No signature-compatible candidates.
            matching_methods.extend(Self::format_candidate_summaries_for_error(
                self.engines,
                &matching_method_decl_refs,
            ));
        }

        // Forward an ErrorRecovery from the first argument if present.
        if let Some(TypeInfo::ErrorRecovery(err)) = arguments_types
            .first()
            .map(|x| (*type_engine.get(*x)).clone())
        {
            return Err(err);
        }

        let type_name = if let Some(call_path) = qualified_call_path {
            format!(
                "{} as {}",
                self.engines.help_out(type_id),
                call_path.call_path
            )
        } else {
            self.engines.help_out(type_id).to_string()
        };

        // Final: MethodNotFound with formatted signature and candidates.
        Err(handler.emit_err(CompileError::MethodNotFound {
            called_method: method_ident.into(),
            expected_signature: format!(
                "{}({}){}",
                method_ident.clone(),
                arguments_types
                    .iter()
                    .map(|a| self.engines.help_out(a).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                if matches!(
                    *self.engines.te().get(self.type_annotation()),
                    TypeInfo::Unknown
                ) {
                    "".to_string()
                } else {
                    format!(" -> {}", self.engines.help_out(self.type_annotation()))
                }
            ),
            type_name,
            matching_methods: matching_methods.into_iter().sorted().collect(),
        }))
    }
}
