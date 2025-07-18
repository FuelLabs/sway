#![allow(dead_code)]
#![allow(deprecated)]

use std::vec;

use crate::{
    internal_error,
    matching::{
        item_impl, lexed_match, lexed_match_mut, literal, ty_match, with_name_mut,
        LexedLocateAnnotatedMut, LexedLocateAsAnnotatedMut,
    },
    migrations::{
        visit_all_modules, visit_all_modules_mut, visit_modules, InteractionResponse,
        MutProgramInfo, Occurrence,
    },
    modifying::*,
    print_single_choice_menu,
};
use anyhow::{bail, Ok, Result};
use itertools::Itertools;
use sway_ast::{ItemKind, Module};
use sway_core::{
    language::{ty::TyModule, CallPath},
    Engines,
};
use sway_error::formatting::{plural_s, Indent};
use sway_types::{Ident, Span, Spanned};

use super::{ContinueMigrationProcess, DryRun, MigrationStep, MigrationStepKind, ProgramInfo};

// NOTE: We do not support cases when `Eq` is given another name via `as` alias import.
//       In practice, this does not happen.

// NOTE: We do not support cases when `Eq` is implemented locally within a function.
//       In practice, this does not happen.

// NOTE: We are searching only for standalone `#[cfg(experimental_partial_eq)]` attributes.
//       For those, we can assume that they are generated using the migration tool, or that
//       early adoption wasn't using complex patterns that request additional effort in
//       migration steps.
//       E.g., if we encounter something like:
//         #[allow(dead_code), cfg(experimental_references = true, experimental_partial_eq = true)]
//       we will assume that developers want to have control over the feature adoption and
//       will not consider such usages in the migration.

// NOTE: We could add an additional migration step that suggests inspecting usages of
//       the `Eq` trait in trait constraints, to see if they could be replaced with `PartialEq`.
//       The development effort of this additional step is questionable. We would need to extend
//       visitors to collect all trait constraints, which is a considerable effort. On the other
//       hand the current types that are constrained all have `Eq` semantics, which is not
//       changed by the introduction of `PartialEq`. Changing `Eq` constraint to `PartialEq`
//       to lower the constraint is done in the `std`, where appropriate.
//       Suggesting to developers doing this replacement in their projects is mentioned
//       in the tracking issue: https://github.com/FuelLabs/sway/issues/6883.

pub(super) const IMPLEMENT_EXPERIMENTAL_PARTIAL_EQ_AND_EQ_TRAITS: MigrationStep = MigrationStep {
    title: "Implement experimental `PartialEq` and `Eq` traits",
    duration: 1,
    kind: MigrationStepKind::CodeModification(
        implement_experimental_partial_eq_and_eq_traits,
        &[],
        // This is an intermediate migration for early adopting the feature.
        ContinueMigrationProcess::Never,
    ),
    help: &[
        "Migration will implement `PartialEq` and `Eq` traits for every type",
        "that implements `Eq` trait.",
        " ",
        "The new implementations will be marked as `#[cfg(experimental_partial_eq = true)]`.",
        " ",
        "The original `Eq` implementation will remain in code and be marked as",
        "`#[cfg(experimental_partial_eq = false)]`.",
    ],
};

pub(super) const REMOVE_DEPRECATED_EQ_TRAIT_IMPLEMENTATIONS: MigrationStep = MigrationStep {
    title: "Remove deprecated `Eq` trait implementations and `experimental_partial_eq` attributes",
    duration: 1,
    kind: MigrationStepKind::Interaction(
        remove_deprecated_eq_trait_implementations_instruction,
        remove_deprecated_eq_trait_implementations_interaction,
        &[],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "Migration will:",
        "  - remove deprecated `Eq` implementations.",
        "  - remove the `#[cfg(experimental_partial_eq = true)]` attributes from the new `Eq`",
        "    and `PartialEq` implementations.",
        " ",
        "Run this migration only if you are switching fully to the `partial_eq` feature,",
        "and do not plan to use the old `Eq` implementations anymore.",
    ],
};

const EXPERIMENTAL_PARTIAL_EQ_ATTRIBUTE: &str = "experimental_partial_eq";

fn remove_deprecated_eq_trait_implementations_instruction(
    program_info: &ProgramInfo,
) -> Result<Vec<Occurrence>> {
    fn remove_deprecated_eq_trait_implementations_instruction_impl(
        _engines: &Engines,
        module: &Module,
        _ty_module: &TyModule,
        _dry_run: DryRun,
    ) -> Result<Vec<Occurrence>> {
        let mut result = vec![];
        // Note that the typed program, depending if the `forc migrate` was run with or
        // without the `partial_eq` flag, might or might not have the deprecated implementations
        // represented in the typed tree.
        // This is not an issue, because if, in the lexed tree, we find a trait impl of the trait
        // named `Eq` that has the `#[cfg(experimental_partial_eq = false)]` attribute, this is
        // enough to identify it as a deprecated `Eq` trait implementation.

        // The deprecated `Eq` implementation:
        // - has the `eq` method in the body, so it must not be empty,
        // - is annotated with `#[cfg(experimental_partial_eq = false)]`.
        result.append(
            &mut find_trait_impl(
                module,
                "Eq",
                false,
                false,
                ResultSpanSource::ImplTraitDefinition,
            )
            .iter()
            .map(|span| span.clone().into())
            .collect(),
        );

        // The new `Eq` implementation:
        // - has an empty impl.
        // - is annotated with `#[cfg(experimental_partial_eq = true)]`.
        result.append(
            &mut find_trait_impl(module, "Eq", true, true, ResultSpanSource::CfgAttribute)
                .iter()
                .map(|span| span.clone().into())
                .collect(),
        );

        // The new `PartialEq` implementation:
        // - has the `eq` method in the body, so it must not be empty,
        // - is annotated with `#[cfg(experimental_partial_eq = true)]`.
        result.append(
            &mut find_trait_impl(
                module,
                "PartialEq",
                false,
                true,
                ResultSpanSource::CfgAttribute,
            )
            .iter()
            .map(|span| span.clone().into())
            .collect(),
        );

        Ok(result)
    }

    let res = visit_all_modules(
        program_info,
        DryRun::Yes,
        remove_deprecated_eq_trait_implementations_instruction_impl,
    )?;

    Ok(res.into_iter().flatten().collect())
}

fn remove_deprecated_eq_trait_implementations_interaction(
    program_info: &mut MutProgramInfo,
) -> Result<(InteractionResponse, Vec<Occurrence>)> {
    /// Calculates and returns:
    /// - number of deprecated `Eq` impls to remove,
    /// - number of `#[cfg(experimental_partial_eq = true)]` to remove from new `Eq` impls,
    /// - number of `#[cfg(experimental_partial_eq = true)]` to remove from new `PartialEq` impls.
    fn calculate_number_of_code_removals(
        _engines: &Engines,
        module: &Module,
        _ty_module: &TyModule,
        _dry_run: DryRun,
    ) -> Result<Vec<(usize, usize, usize)>> {
        // We will conveniently reuse the `find_trait_impl` function here.
        let num_of_deprecated_eq_impls = find_trait_impl(
            module,
            "Eq",
            false,
            false,
            ResultSpanSource::ImplTraitDefinition,
        )
        .len();

        let num_of_cfg_attrs_on_new_eq =
            find_trait_impl(module, "Eq", true, true, ResultSpanSource::CfgAttribute).len();

        let num_of_cfg_attrs_on_new_partial_eq = find_trait_impl(
            module,
            "PartialEq",
            false,
            true,
            ResultSpanSource::CfgAttribute,
        )
        .len();

        Ok(vec![(
            num_of_deprecated_eq_impls,
            num_of_cfg_attrs_on_new_eq,
            num_of_cfg_attrs_on_new_partial_eq,
        )])
    }

    let numbers_of_code_removals_per_module = visit_modules(
        program_info.engines,
        &program_info.lexed_program.root,
        &program_info.ty_program.root_module,
        DryRun::Yes,
        calculate_number_of_code_removals,
    )?;

    let (
        num_of_deprecated_eq_impls,
        num_of_cfg_attrs_on_new_eq,
        num_of_cfg_attrs_on_new_partial_eq,
    ) = numbers_of_code_removals_per_module
        .into_iter()
        .flatten()
        .fold((0, 0, 0), |acc, counts| {
            (acc.0 + counts.0, acc.1 + counts.1, acc.2 + counts.2)
        });

    if num_of_deprecated_eq_impls == 0
        && num_of_cfg_attrs_on_new_eq == 0
        && num_of_cfg_attrs_on_new_partial_eq == 0
    {
        return Ok((InteractionResponse::None, vec![]));
    }

    println!("The following code will be removed:");
    if num_of_deprecated_eq_impls > 0 {
        println!(
            "{}- {} deprecated `Eq` implementation{}.",
            Indent::Single,
            num_of_deprecated_eq_impls,
            plural_s(num_of_deprecated_eq_impls)
        );
    }
    if num_of_cfg_attrs_on_new_eq > 0 {
        println!("{}- {} `#[cfg(experimental_partial_eq = true)]` attributes from new `Eq` implementation{}.", Indent::Single, num_of_cfg_attrs_on_new_eq, plural_s(num_of_cfg_attrs_on_new_eq));
    }
    if num_of_cfg_attrs_on_new_partial_eq > 0 {
        println!("{}- {} `#[cfg(experimental_partial_eq = true)]` attributes from new `PartialEq` implementation{}.", Indent::Single, num_of_cfg_attrs_on_new_partial_eq, plural_s(num_of_cfg_attrs_on_new_partial_eq));
    }
    println!();
    println!("Do you want to remove that code and switch fully to the `partial_eq` feature?");
    println!();

    if print_single_choice_menu(&[
        "Yes, remove the code and switch fully to the `partial_eq` feature.",
        "No, continue using deprecated `Eq` and the new `PartialEq` and `Eq` side-by-side.",
    ]) != 0
    {
        return Ok((InteractionResponse::PostponeStep, vec![]));
    }

    // Execute the migration step.
    let mut result = vec![];

    fn remove_deprecated_eq_impls(
        _engines: &Engines,
        module: &mut Module,
        _ty_module: &TyModule,
        _dry_run: DryRun,
    ) -> Result<Vec<Occurrence>> {
        let mut occurrences_of_annotated_eq_impls_to_remove: Vec<Occurrence> = vec![];

        // Deprecated `Eq` impls must not be empty, they have the `eq` method.
        let annotated_eq_impls =
            lexed_match::impl_self_or_trait_decls_annotated(module).filter(|annotated| {
                matches!(&annotated.value,
                    ItemKind::Impl(item_impl)
                        if item_impl::implements_trait("Eq")(&item_impl)
                            && !item_impl.contents.inner.is_empty()
                )
            });

        // For every empty `Eq` trait implementation...
        for annotated_eq_impl in annotated_eq_impls {
            // Check if the `#[cfg(experimental_partial_eq = false)]` attribute exists.
            if lexed_match::cfg_attribute_standalone_single_arg(
                &annotated_eq_impl.attributes,
                EXPERIMENTAL_PARTIAL_EQ_ATTRIBUTE,
                |arg| arg.as_ref().is_some_and(literal::is_bool_false),
            )
            .is_none()
            {
                continue;
            };

            // The trait impl passes all conditions, mark it for removal.
            occurrences_of_annotated_eq_impls_to_remove.push(annotated_eq_impl.span().into());
        }

        for annotated_eq_impl_occurrence in occurrences_of_annotated_eq_impls_to_remove.iter() {
            modify(module).remove_annotated_item(&annotated_eq_impl_occurrence.span);
        }

        Ok(occurrences_of_annotated_eq_impls_to_remove)
    }

    let res = visit_all_modules_mut(program_info, DryRun::No, remove_deprecated_eq_impls)?;

    result.append(&mut res.into_iter().flatten().collect());

    fn remove_cfg_experimental_partial_eq_true_attributes(
        _engines: &Engines,
        module: &mut Module,
        _ty_module: &TyModule,
        _dry_run: DryRun,
    ) -> Result<Vec<Occurrence>> {
        let mut occurrences_of_cfg_attributes_to_remove: Vec<Occurrence> = vec![];

        // Find new `Eq` and `PartialEq` impls.
        let annotated_trait_impls = lexed_match_mut::impl_self_or_trait_decls_annotated(module)
            .filter_map(|annotated|
                 if matches!(&annotated.value,
                    ItemKind::Impl(item_impl)
                        // New `Eq` impl must be empty, and `PartialEq` not, it has the `eq` method.
                        if item_impl::implements_trait("Eq")(&item_impl) && item_impl.contents.inner.is_empty() ||
                           item_impl::implements_trait("PartialEq")(&item_impl) && !item_impl.contents.inner.is_empty())
                    {
                        Some(annotated)
                    } else {
                        None
                    }
            )
            .collect_vec();

        // For every `Eq` and `PartialEq` trait implementation...
        for annotated_trait_impl in annotated_trait_impls.iter() {
            // Check if the `#[cfg(experimental_partial_eq = true)]` attribute exists.
            let Some(cfg_experimental_partial_eq_attr) =
                lexed_match::cfg_attribute_standalone_single_arg(
                    &annotated_trait_impl.attributes,
                    EXPERIMENTAL_PARTIAL_EQ_ATTRIBUTE,
                    |arg| arg.as_ref().is_some_and(literal::is_bool_true),
                )
            else {
                continue;
            };

            // The trait passes all the conditions, mark the `cfg` attribute for removal.
            occurrences_of_cfg_attributes_to_remove
                .push(cfg_experimental_partial_eq_attr.span().into());
        }

        for annotated_trait_impl in annotated_trait_impls {
            for cfg_attribute_occurrence in occurrences_of_cfg_attributes_to_remove.iter() {
                modify(annotated_trait_impl)
                    .remove_attribute_decl_containing_attribute(&cfg_attribute_occurrence.span);
            }
        }

        Ok(occurrences_of_cfg_attributes_to_remove)
    }

    let res = visit_all_modules_mut(
        program_info,
        DryRun::No,
        remove_cfg_experimental_partial_eq_true_attributes,
    )?;

    result.append(&mut res.into_iter().flatten().collect());

    Ok((InteractionResponse::ExecuteStep, result))
}

enum ResultSpanSource {
    ImplTraitDefinition,
    CfgAttribute,
}

/// Searches for impls of the trait named `trait_name` within the `module`.
/// The trait impl must either be empty, or have content, depending on `is_empty_impl`.
/// The trait impl must have the `#[cfg(experimental_partial_eq)]` set to bool defined in `cfg_experimental_partial_eq`.
/// The resulting [Span] points either to the trait impl definition (without the where clause and the content),
/// or to the `cfg` attribute.
fn find_trait_impl(
    module: &Module,
    trait_name: &str,
    is_empty_impl: bool,
    cfg_experimental_partial_eq: bool,
    result_span_source: ResultSpanSource,
) -> Vec<Span> {
    let mut result = vec![];

    // Find impls of the trait given by the `trait_name`.
    let attributed_eq_trait_impls = lexed_match::impl_self_or_trait_decls_annotated(module)
        .filter_map(|annotated| match &annotated.value {
            ItemKind::Impl(item_impl) if item_impl::implements_trait(trait_name)(&item_impl) => {
                Some((&annotated.attributes, item_impl))
            }
            _ => None,
        });

    // For every trait implementation...
    for (attributes, eq_trait_impl) in attributed_eq_trait_impls {
        // Check if the impl body is empty or not, and same as expected.
        if eq_trait_impl.contents.inner.is_empty() != is_empty_impl {
            continue;
        }

        // Check if the `#[cfg(experimental_partial_eq)]` attribute exists and is set to `cfg_experimental_partial_eq`.
        let expected_bool_literal = if cfg_experimental_partial_eq {
            literal::is_bool_true
        } else {
            literal::is_bool_false
        };

        let Some(cfg_experimental_partial_eq_attr) =
            lexed_match::cfg_attribute_standalone_single_arg(
                attributes,
                EXPERIMENTAL_PARTIAL_EQ_ATTRIBUTE,
                |arg| arg.as_ref().is_some_and(expected_bool_literal),
            )
        else {
            continue;
        };

        // The trait passes all the conditions, add it to the result.
        let result_span = match result_span_source {
            ResultSpanSource::ImplTraitDefinition => {
                Span::join(eq_trait_impl.impl_token.span(), &eq_trait_impl.ty.span())
            }
            ResultSpanSource::CfgAttribute => cfg_experimental_partial_eq_attr.span(),
        };
        result.push(result_span);
    }

    result
}

fn implement_experimental_partial_eq_and_eq_traits(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
) -> Result<Vec<Occurrence>> {
    fn implement_experimental_partial_eq_and_eq_traits_impl(
        engines: &Engines,
        lexed_module: &mut Module,
        ty_module: &TyModule,
        dry_run: DryRun,
    ) -> Result<Vec<Occurrence>> {
        let mut result = vec![];

        let std_ops_eq_call_path = CallPath::fullpath(&["std", "ops", "Eq"]);

        let ty_impl_traits = ty_match::impl_self_or_trait_decls(ty_module)
            .map(|decl| engines.de().get_impl_self_or_trait(decl))
            .filter(|decl| decl.is_impl_trait())
            .collect_vec();

        for ty_impl_trait in ty_impl_traits {
            let implemented_trait = engines.de().get_trait(
                &ty_impl_trait
                    .implemented_trait_decl_id()
                    .expect("impl is a trait impl"),
            );
            // Further inspect only `Eq` impls.
            if implemented_trait.call_path != std_ops_eq_call_path {
                continue;
            }

            let Some((attributes, lexed_impl_eq_trait)) =
                lexed_module.locate_annotated_mut(&ty_impl_trait)
            else {
                bail!(internal_error(
                    "Lexical trait implementation cannot be found."
                ));
            };

            // If the impl already has `experimental_partial_eq` attribute set, we assume that the migration
            // is already done for this impl. Note that we don't check if it is set to true or false.
            // Just the existence of the attribute, being on the old `Eq` (false), or the new `Eq` (true),
            // indicates that the `partial_eq` migrations are done for this occurrence, or that developers
            // manually early adopted the feature.
            if lexed_match_mut::cfg_attribute_arg(
                attributes,
                with_name_mut(EXPERIMENTAL_PARTIAL_EQ_ATTRIBUTE),
            )
            .is_some()
            {
                continue;
            };

            // Check that this is the old `Eq` implementation, with the `eq` method.
            // If it is the new, empty one, skip it.
            if lexed_impl_eq_trait.contents.inner.is_empty() {
                continue;
            }

            result.push(
                Span::join(
                    lexed_impl_eq_trait.impl_token.span(),
                    &lexed_impl_eq_trait.ty.span(),
                )
                .into(),
            );

            if dry_run == DryRun::Yes {
                continue;
            }

            // No dry run, perform the changes.

            // 1. Append the `cfg[experimental_partial_eq = false]` to the existing attributes.
            let insert_span = if attributes.is_empty() {
                Span::empty_at_start(&lexed_impl_eq_trait.span())
            } else {
                Span::empty_at_end(&attributes.last().expect("attributes are not empty").span())
            };

            let cfg_attribute_decl =
                New::cfg_experimental_attribute_decl(insert_span.clone(), "partial_eq", false);

            attributes.push(cfg_attribute_decl);

            // 2. Insert the `PartialEq` and new empty `Eq` implementation.

            let Some(annotated_impl_eq_trait) =
                lexed_module.locate_as_annotated_mut(&ty_impl_trait)
            else {
                bail!(internal_error(
                    "Annotated lexical trait implementation cannot be found."
                ));
            };

            let mut annotated_impl_partial_eq_trait = annotated_impl_eq_trait.clone();

            // Set the `experimental_partial_eq` attribute to true.
            let Some(experimental_partial_eq_arg) = lexed_match_mut::cfg_attribute_arg(
                &mut annotated_impl_partial_eq_trait.attributes,
                with_name_mut(EXPERIMENTAL_PARTIAL_EQ_ATTRIBUTE),
            ) else {
                bail!(internal_error(
                    "Attribute \"experimental_partial_eq\" cannot be found."
                ));
            };
            experimental_partial_eq_arg.value = Some(New::literal_bool(insert_span, true));

            // Define the new `Eq` trait simply by removing the content form the `PartialEq` trait.
            let mut annotated_impl_new_eq_trait = annotated_impl_partial_eq_trait.clone();
            let ItemKind::Impl(impl_new_eq_trait) = &mut annotated_impl_new_eq_trait.value else {
                bail!(internal_error(
                    "Annotated implementation of \"Eq\" trait is not an `Item::Impl`."
                ));
            };
            impl_new_eq_trait.contents.inner.clear();

            // Rename the `Eq` to `PartialEq` in the new `PartialEq` trait.
            let ItemKind::Impl(impl_partial_eq_trait) = &mut annotated_impl_partial_eq_trait.value
            else {
                bail!(internal_error(
                    "Annotated implementation of \"Eq\" trait is not an `Item::Impl`."
                ));
            };

            let path_type_last_ident = impl_partial_eq_trait
                .trait_opt
                .as_mut()
                .expect("impl implements `Eq` trait")
                .0
                .last_segment_mut();
            path_type_last_ident.name =
                Ident::new_with_override("PartialEq".into(), path_type_last_ident.name.span());

            // If the original `Eq` impl had `Eq`s in trait constraints, replace those with `PartialEq`.
            let eq_trait_constraints =
                lexed_match_mut::trait_constraints(impl_partial_eq_trait, with_name_mut("Eq"));
            for eq_trait_constraint in eq_trait_constraints {
                let path_type_last_ident = eq_trait_constraint.last_segment_mut();
                path_type_last_ident.name =
                    Ident::new_with_override("PartialEq".into(), path_type_last_ident.name.span());
            }

            // Add the new trait impls to the items.
            // let mut module_modifier = Modifier::new(lexed_module);
            // module_modifier
            modify(lexed_module)
                // Inserting in reverse order so that `PartialEq` ends up before `Eq`,
                // since they have the same span start which equals the span of the original `Eq`.
                .insert_annotated_item_after(annotated_impl_new_eq_trait)
                .insert_annotated_item_after(annotated_impl_partial_eq_trait);

            // Note that we do not need to adjust the `use` statements to include `PartialEq`.
            // All `std::ops` are a part of the std's prelude. If there was a `use Eq`
            // in a modified file, it was actually not needed.
        }

        Ok(result)
    }

    let res = visit_all_modules_mut(
        program_info,
        dry_run,
        implement_experimental_partial_eq_and_eq_traits_impl,
    )?;

    Ok(res.into_iter().flatten().collect())
}
