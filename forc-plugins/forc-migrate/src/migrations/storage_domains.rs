#![allow(dead_code)]

use std::collections::HashSet;

use super::{ContinueMigrationProcess, MigrationStep, MigrationStepKind, MutProgramInfo};
use crate::{
    internal_error,
    matching::{
        lexed_match_mut, lexed_storage_field, ty_match,
        ty_storage_field::{with_in_keyword, without_in_keyword},
        TyLocate,
    },
    migrations::{InteractionResponse, ProgramInfo},
    modifying::*,
    print_single_choice_menu,
};
use anyhow::{bail, Ok, Result};
use itertools::Itertools;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use sway_core::language::{
    ty::{TyExpressionVariant, TyStorageField},
    CallPath, Literal,
};
use sway_error::formatting::{self, sequence_to_list};
use sway_types::{Span, Spanned};

pub(super) const REVIEW_STORAGE_SLOT_KEYS_STEP: MigrationStep = MigrationStep {
    title: "Review explicitly defined slot keys in storage declarations (`in` keywords)",
    duration: 2,
    kind: MigrationStepKind::Instruction(review_storage_slot_keys_step),
    help: &[
        "If the slot keys used in `in` keywords represent keys generated for `storage` fields",
        "by the Sway compiler, those keys might need to be recalculated.",
        " ",
        "The previous formula for calculating storage keys was: `sha256(\"storage.<field name>\")`.",
        "The new formula is:                                    `sha256((0u8, \"storage.<field name>\"))`.",
    ],
};

pub(super) const DEFINE_BACKWARD_COMPATIBLE_STORAGE_SLOT_KEYS_STEP: MigrationStep = MigrationStep {
    title: "Explicitly define storage slot keys if they need to be backward compatible",
    // We will be pointing to the storage declaration and offer automatic migration.
    // In case of a suggestion the manual effort will be reviewing the purpose of the
    // contract, which we will approximate with 10 minutes.
    duration: 10,
    kind: MigrationStepKind::Interaction(
        define_backward_compatible_storage_slot_keys_step_instruction,
        define_backward_compatible_storage_slot_keys_step_interaction,
        &[],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "If the contract owning this storage is behind a proxy, or for any other reason needs",
        "to use previous storage slot keys, those keys must be explicitly assigned to the",
        "storage fields by using the `in` keyword.",
        " ",
        "E.g.:",
        "    storage {",
        "        field in <previous slot key>: u64 = 0,",
        "    }",
        " ",
        "The previous formula for calculating storage keys was: `sha256(\"storage.<field name>\")`.",
        "The new formula is:                                    `sha256((0u8, \"storage.<field name>\"))`.",
    ],
};

fn review_storage_slot_keys_step(program_info: &ProgramInfo) -> Result<Vec<Span>> {
    let mut res = vec![];

    let Some(storage_decl_id) = ty_match::storage_decl(program_info.ty_program.as_ref()) else {
        return Ok(res);
    };

    let storage_decl = &*program_info.engines.de().get_storage(&storage_decl_id);

    let well_known_slot_keys = get_well_known_slot_keys();
    let well_known_slot_keys_constants = get_well_known_slot_keys_constants();

    for (storage_field, key_expression) in
        ty_match::storage_fields_deep(storage_decl, with_in_keyword)
            .iter()
            .map(|sf| {
                (
                    sf,
                    sf.key_expression
                        .as_ref()
                        .expect("storage key has in keyword"),
                )
            })
    {
        // If the key expression represents a well known slot defined in
        // Sway Standards or Sway Libraries do not suggest to check it.
        let is_well_known_slot_key = match &key_expression.expression {
            TyExpressionVariant::Literal(Literal::B256(slot_key)) => {
                well_known_slot_keys.contains(&BigUint::from_bytes_be(slot_key.as_slice()))
            }
            TyExpressionVariant::ConstantExpression {
                call_path: Some(call_path),
                ..
            } => well_known_slot_keys_constants.contains(call_path),
            _ => false,
        };
        if is_well_known_slot_key {
            continue;
        }

        // If the storage fields are behind a proxy, and must contain the backwards compatibility,
        // the next migration, will assign them the slots calculated by the previous algorithm.
        //
        // If we see that the `in` keyword assigns a literal that corresponds to the slot calculated
        // by the previous algorithm, we recognize it as backwards compatibility and do not suggest to
        // review the slot.
        let is_backward_compatibility_slot_key = match &key_expression.expression {
            TyExpressionVariant::Literal(Literal::B256(slot_key)) => {
                slot_key == &get_previous_slot_key(storage_field)
            }
            _ => false,
        };
        if is_backward_compatibility_slot_key {
            continue;
        }

        res.push(key_expression.span.clone());
    }

    Ok(res)
}

fn define_backward_compatible_storage_slot_keys_step_instruction(
    program_info: &ProgramInfo,
) -> Result<Vec<Span>> {
    let mut res = vec![];

    let Some(storage_decl_id) = ty_match::storage_decl(program_info.ty_program.as_ref()) else {
        return Ok(res);
    };

    let storage_decl = &*program_info.engines.de().get_storage(&storage_decl_id);

    // It is hard to have any better heuristic here. Essentially, every contract
    // could be behind a proxy and we do not have a mean to detected that.
    // So, we will provide the suggestion if the storage has any fields without the `in` keyword.
    // The suggestion is shown only once on the entire `storage` declaration,
    // to avoid cluttering. The interaction part of the step will then provide
    // more detailed information and guide the developers.
    if !ty_match::storage_fields_deep(storage_decl, without_in_keyword).is_empty() {
        res.push(storage_decl.span.clone());
    }

    Ok(res)
}

fn define_backward_compatible_storage_slot_keys_step_interaction(
    program_info: &mut MutProgramInfo,
) -> Result<(InteractionResponse, Vec<Span>)> {
    let Some(storage_decl_id) = ty_match::storage_decl(program_info.ty_program) else {
        return Ok((InteractionResponse::None, vec![]));
    };

    let storage_decl = &*program_info.engines.de().get_storage(&storage_decl_id);

    let storage_fields_without_in_keyword =
        ty_match::storage_fields_deep(storage_decl, without_in_keyword);

    println!(
        "The following storage fields will have slot keys calculated by using the new formula:"
    );
    sequence_to_list(
        &storage_fields_without_in_keyword
            .iter()
            .map(|field| field.full_name())
            .collect_vec(),
        formatting::Indent::Single,
        10,
    )
    .iter()
    .for_each(|field_full_name| println!("{field_full_name}"));
    println!();
    println!("Do you want these fields to have backward compatible storage slot keys, calculated");
    println!("by using the previous formula?");
    println!();
    println!("If yes, this migration step will insert `in` keywords to all of the above fields,");
    println!("and calculate the storage slot keys by using the previous formula.");
    println!();

    if print_single_choice_menu(&[
        "Yes, assign the backward compatible storage slot keys.",
        "No, this contract does not require backwards compatibility.",
    ]) != 0
    {
        return Ok((InteractionResponse::StepNotNeeded, vec![]));
    }

    // Execute the migration step.
    let mut res = vec![];

    let Some(storage_declaration) = lexed_match_mut::storage_decl(program_info.lexed_program)
    else {
        bail!(internal_error(
            "Lexical storage declaration cannot be found."
        ));
    };

    for lexed_storage_field in lexed_match_mut::storage_fields_deep(
        storage_declaration,
        lexed_storage_field::without_in_keyword,
    ) {
        let Some(ty_storage_field) = storage_decl.locate(lexed_storage_field) else {
            bail!(internal_error(format!(
                "Typed storage field \"{}\" cannot be found.",
                lexed_storage_field.name
            )));
        };

        res.push(ty_storage_field.name.span());

        modify(lexed_storage_field).set_in_key(BigUint::from_bytes_be(
            get_previous_slot_key(ty_storage_field).as_slice(),
        ));
    }

    Ok((InteractionResponse::ExecuteStep, res))
}

/// Returns storage slot keys defined in Sway Standards and Sway Libraries,
/// as [BigUint]s that represents `b256` storage addresses.
fn get_well_known_slot_keys() -> HashSet<BigUint> {
    // For SRC14 well-known slot keys see: https://docs.fuel.network/docs/sway-libs/upgradability/#upgradability-library
    let src14_target = BigUint::parse_bytes(
        b"7bb458adc1d118713319a5baa00a2d049dd64d2916477d2688d76970c898cd55",
        16,
    )
    .unwrap();
    let src14_proxy_owner = BigUint::parse_bytes(
        b"bb79927b15d9259ea316f2ecb2297d6cc8851888a98278c0a2e03e1a091ea754",
        16,
    )
    .unwrap();

    HashSet::from_iter(vec![src14_target, src14_proxy_owner])
}

/// Returns [CallPath]s of constants that hold storage slot keys
/// defined in Sway Standards and Sway Libraries.
fn get_well_known_slot_keys_constants() -> HashSet<CallPath> {
    let slot_keys_constants = vec![
        // For SRC14 well-known slot keys see: https://docs.fuel.network/docs/sway-libs/upgradability/#upgradability-library
        ["sway_libs", "upgradability", "PROXY_OWNER_STORAGE"],
        ["standards", "src14", "SRC14_TARGET_STORAGE"],
    ]
    .into_iter()
    .map(|path_parts| CallPath::fullpath(&path_parts));

    HashSet::from_iter(slot_keys_constants)
}

fn get_previous_slot_key(storage_field: &TyStorageField) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(storage_field.full_name());
    hasher.finalize().into()
}
