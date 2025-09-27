use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;

use crate::{
    engine_threading::{GetCallPathWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::CallPath,
    Engines, IncludeSelf, SubstTypes, SubstTypesContext, TypeId, TypeInfo, TypeParameter,
    TypeSubstMap, UnifyCheck,
};

use super::{Package, TraitMap};

// Given an impl of the form `impl<P1..=Pn> Trait<T1..=Tn>` for `T0`, the impl is allowed if:
//  1. Trait is a local trait
//    or
//  2. All of
//      a) At least one of the types `T0..=Tn` must be a local type. Let Ti be the first such type.
//      b) No uncovered type parameters `P1..=Pn` may appear in `T0..Ti` (excluding `Ti`)
//         This is already checked elsewhere in the compiler so no need to check here.
pub(crate) fn check_orphan_rules_for_impls(
    handler: &Handler,
    engines: &Engines,
    current_package: &Package,
) -> Result<(), ErrorEmitted> {
    let mut error: Option<ErrorEmitted> = None;
    let module = &current_package.root_module();
    module.walk_scope_chain(|lexical_scope| {
        let trait_map = &lexical_scope.items.implemented_traits;
        if let Err(err) =
            check_orphan_rules_for_impls_in_scope(handler, engines, current_package, trait_map)
        {
            error = Some(err);
        }
    });

    match error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

fn check_orphan_rules_for_impls_in_scope(
    handler: &Handler,
    engines: &Engines,
    current_package: &Package,
    trait_map: &TraitMap,
) -> Result<(), ErrorEmitted> {
    for key in trait_map.trait_impls.keys() {
        for trait_entry in trait_map.trait_impls[key].iter() {
            // 0. If it's a contract then skip it as it's not relevant to coherence.
            if engines
                .te()
                .get(trait_entry.inner.key.type_id)
                .is_contract()
            {
                continue;
            }

            // 1. Check if trait is local to the current package
            let package_name = trait_entry.inner.key.name.prefixes.first().unwrap();

            let package_program_id = current_package.program_id();

            let trait_impl_program_id = match trait_entry.inner.value.impl_span.source_id() {
                Some(source_id) => source_id.program_id(),
                None => {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Expected a valid source id",
                        trait_entry.inner.value.impl_span.clone(),
                    )))
                }
            };

            if package_program_id != trait_impl_program_id {
                continue;
            }

            if package_name == current_package.name() {
                continue;
            }

            fn references_local_type(
                engines: &Engines,
                current_package: &Package,
                type_id: TypeId,
            ) -> bool {
                // Create a flag to track if a local type was foundt.
                let found_local = Cell::new(false);

                type_id.walk_inner_types(
                    engines,
                    IncludeSelf::Yes,
                    &|inner_type_id| {
                        // If we've already flagged a local type, no need to do further work.
                        if found_local.get() {
                            return;
                        }

                        let inner_type = engines.te().get(*inner_type_id);
                        let is_local = match *inner_type {
                            TypeInfo::Enum(decl_id) => {
                                let enum_decl = engines.de().get_enum(&decl_id);
                                is_from_local_package(current_package, &enum_decl.call_path)
                            }
                            TypeInfo::Struct(decl_id) => {
                                let struct_decl = engines.de().get_struct(&decl_id);
                                is_from_local_package(current_package, &struct_decl.call_path)
                            }
                            // FIXME: We treat arrays as a special case for now due to lack of const generics.
                            TypeInfo::Array(_, _) => true,
                            TypeInfo::StringArray(_) => true,
                            _ => false,
                        };

                        // Mark the flag if a local type is found.
                        if is_local {
                            found_local.set(true);
                        }
                    },
                    &|trait_constraint| {
                        // If we've already flagged a local type, no need to do further work.
                        if found_local.get() {
                            return;
                        }

                        let is_local =
                            is_from_local_package(current_package, &trait_constraint.trait_name);

                        // Mark the flag if a local type is found.
                        if is_local {
                            found_local.set(true);
                        }
                    },
                );

                found_local.get()
            }

            // 2. Now the trait is necessarily upstream to the current package
            let mut has_local_type = false;
            for arg in &trait_entry.inner.key.name.suffix.args {
                has_local_type |= references_local_type(engines, current_package, arg.type_id());
                if has_local_type {
                    break;
                }
            }

            'tp: for type_id in &trait_entry.inner.key.impl_type_parameters {
                let tp = engines.te().get(*type_id);
                match tp.as_ref() {
                    TypeInfo::TypeParam(tp) => match tp {
                        TypeParameter::Type(tp) => {
                            for tc in &tp.trait_constraints {
                                has_local_type |=
                                    is_from_local_package(current_package, &tc.trait_name);
                                if has_local_type {
                                    break 'tp;
                                }
                            }
                        }
                        TypeParameter::Const(_tp) => {}
                    },
                    _ => unreachable!(),
                }
            }

            has_local_type |=
                references_local_type(engines, current_package, trait_entry.inner.key.type_id);

            if !has_local_type {
                let trait_name = trait_entry.inner.key.name.suffix.name.to_string();
                let type_name = {
                    let ty = engines.te().get(trait_entry.inner.key.type_id);
                    ty.get_type_str(engines)
                };
                handler.emit_err(CompileError::IncoherentImplDueToOrphanRule {
                    trait_name,
                    type_name,
                    span: trait_entry.inner.value.impl_span.clone(),
                });
            }
        }
    }
    Ok(())
}

fn is_from_local_package(current_package: &Package, call_path: &CallPath) -> bool {
    let package_name = call_path.prefixes.first().unwrap();
    let is_external =
        current_package
            .external_packages
            .iter()
            .any(|(external_package_name, _root)| {
                external_package_name.as_str() == package_name.as_str()
            });
    if is_external {
        return false;
    }
    assert_eq!(current_package.name().as_str(), package_name.as_str());
    true
}

/// Given [TraitMap]s `self` and `other`, checks for overlaps between `self` and `other`.
/// If no overlaps are found extends `self` with `other`.
pub(crate) fn check_impls_for_overlap(
    trait_map: &mut TraitMap,
    handler: &Handler,
    other: TraitMap,
    engines: &Engines,
) -> Result<(), ErrorEmitted> {
    let mut overlap_err = None;
    let unify_check = UnifyCheck::constraint_subset(engines);
    let mut traits_types = HashMap::<CallPath, HashSet<TypeId>>::new();
    trait_map.get_traits_types(&mut traits_types)?;
    other.get_traits_types(&mut traits_types)?;

    for key in trait_map.trait_impls.keys() {
        for self_entry in trait_map.trait_impls[key].iter() {
            let self_tcs: Vec<(CallPath, TypeId)> = self_entry
                .inner
                .key
                .impl_type_parameters
                .iter()
                .flat_map(|type_id| {
                    let ti = engines.te().get(*type_id);
                    match ti.as_ref() {
                        TypeInfo::TypeParam(tp) => match tp {
                            TypeParameter::Type(tp) => tp
                                .trait_constraints
                                .iter()
                                .map(|tc| (tc.trait_name.clone(), tp.type_id))
                                .collect::<Vec<_>>(),
                            TypeParameter::Const(_tp) => vec![],
                        },
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<_>>();

            let self_call_path = engines
                .te()
                .get(self_entry.inner.key.type_id)
                .call_path(engines);
            other.for_each_impls(engines, self_entry.inner.key.type_id, true, |other_entry| {
                let other_call_path = engines
                    .te()
                    .get(other_entry.inner.key.type_id)
                    .call_path(engines);

                // This prevents us from checking duplicated types as might happen when
                // compiling different versions of the same library.
                let is_duplicated_type = matches!(
                    (&self_call_path, &other_call_path),
                    (Some(v1), Some(v2))
                        if v1.prefixes == v2.prefixes
                            && v1.span().source_id() != v2.span().source_id()
                );

                if self_entry.inner.key.name.eq(
                    &*other_entry.inner.key.name,
                    &PartialEqWithEnginesContext::new(engines),
                ) && self_entry.inner.value.impl_span != other_entry.inner.value.impl_span
                    && !is_duplicated_type
                    && (unify_check
                        .check(self_entry.inner.key.type_id, other_entry.inner.key.type_id)
                        || unify_check
                            .check(other_entry.inner.key.type_id, self_entry.inner.key.type_id))
                {
                    let other_tcs: Vec<(CallPath, TypeId)> = other_entry
                        .inner
                        .key
                        .impl_type_parameters
                        .iter()
                        .flat_map(|type_id| {
                            let ti = engines.te().get(*type_id);
                            match ti.as_ref() {
                                TypeInfo::TypeParam(tp) => match tp {
                                    TypeParameter::Type(tp) => tp
                                        .trait_constraints
                                        .iter()
                                        .map(|tc| (tc.trait_name.clone(), tp.type_id))
                                        .collect::<Vec<_>>(),
                                    TypeParameter::Const(_tp) => vec![],
                                },
                                _ => unreachable!(),
                            }
                        })
                        .collect::<Vec<_>>();
                    let other_tcs_satisfied = other_tcs.iter().all(|(trait_name, tp_type_id)| {
                        if let Some(tc_type_ids) = traits_types.get(trait_name) {
                            tc_type_ids.iter().any(|tc_type_id| {
                                let mut type_mapping = TypeSubstMap::new();
                                type_mapping.insert(*tp_type_id, *tc_type_id);
                                let mut type_id = other_entry.inner.key.type_id;
                                type_id.subst(&SubstTypesContext::new(
                                    engines,
                                    &type_mapping,
                                    false,
                                ));
                                unify_check.check(self_entry.inner.key.type_id, type_id)
                            })
                        } else {
                            false
                        }
                    });

                    let self_tcs_satisfied = self_tcs.iter().all(|(trait_name, tp_type_id)| {
                        if let Some(tc_type_ids) = traits_types.get(trait_name) {
                            tc_type_ids.iter().any(|tc_type_id| {
                                let mut type_mapping = TypeSubstMap::new();
                                type_mapping.insert(*tp_type_id, *tc_type_id);
                                let mut type_id = self_entry.inner.key.type_id;
                                type_id.subst(&SubstTypesContext::new(
                                    engines,
                                    &type_mapping,
                                    false,
                                ));
                                unify_check.check(other_entry.inner.key.type_id, type_id)
                            })
                        } else {
                            false
                        }
                    });

                    if other_tcs_satisfied && self_tcs_satisfied {
                        for trait_item_name1 in self_entry.inner.value.trait_items.keys() {
                            for trait_item_name2 in other_entry.inner.value.trait_items.keys() {
                                if trait_item_name1 == trait_item_name2 {
                                    overlap_err = Some(
                                        handler.emit_err(
                                            CompileError::ConflictingImplsForTraitAndType {
                                                trait_name: engines
                                                    .help_out(self_entry.inner.key.name.as_ref())
                                                    .to_string(),
                                                type_implementing_for: engines
                                                    .help_out(self_entry.inner.key.type_id)
                                                    .to_string(),
                                                type_implementing_for_unaliased: engines
                                                    .help_out(self_entry.inner.key.type_id)
                                                    .to_string(),
                                                existing_impl_span: self_entry
                                                    .inner
                                                    .value
                                                    .impl_span
                                                    .clone(),
                                                second_impl_span: other_entry
                                                    .inner
                                                    .value
                                                    .impl_span
                                                    .clone(),
                                            },
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    if let Some(overlap_err) = overlap_err {
        return Err(overlap_err);
    }

    trait_map.extend(other, engines);

    Ok(())
}
