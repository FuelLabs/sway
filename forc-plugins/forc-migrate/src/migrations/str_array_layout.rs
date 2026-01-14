#![allow(dead_code)]

use super::{MigrationStep, MigrationStepKind};
use crate::{
    migrations::{DryRun, Occurrence, ProgramInfo},
    visiting::{InvalidateTypedElement, ProgramVisitor, TreesVisitor, VisitingContext},
};
use anyhow::{Ok, Result};
use sway_ast::StorageField;
use sway_core::{language::ty::TyStorageField, TypeInfo};
use sway_types::Spanned;

pub(super) const REVIEW_EXISTING_USAGES_OF_STORAGE_STR_ARRAY: MigrationStep = MigrationStep {
    title: "Review storage of string arrays",
    duration: 10,
    kind: MigrationStepKind::Instruction(review_existing_usages_of_storage_and_str_array),
    help: &[
        "The runtime layout of string arrays is changing.",
        " ",
        "The current behaviour pads their sizes until they reach a multiple of 8.",
        "This was done as an optimisation for `encoding v0`, but given that it is",
        "reaching its end of life, this behaviour will be kept only when the ",
        "experimental compiler flag `str_array_no_padding` is set to `false`.",
        " ",
        "For `encoding v1`, the compiler can achieve better performance by removing ",
        "this padding, allowing more types to be trivially encodable/decodable, and",
        "triggering other optimisations. This will be the new behaviour when ",
        "`str_array_no_padding` is set to `true`.",
        " ",
        "This difference directly impacts storage, because with the new layout, reads",
        "and writes of any type containing string arrays will touch different slots. ",
        "That happens because their size will be different.",
        " ",
        "One solution when migrating data is to read from storage using a version of",
        "the `struct` that instead of `str[N]` has `[u8; M]`, where M is the next ",
        "multiple of 8 after N, simulating the padding.",
        " ",
        "╔═════════════════════════════════════════════════════════════════════════════════════╗",
        "║ The above occurrences must not be seen as comprehensive, but rather as a guideline. ║",
        "║ Carefully review all the storage access in your code.                               ║", 
        "╚═════════════════════════════════════════════════════════════════════════════════════╝",
    ],
};

// NOTE: When analyzing storage fields, we expect that the storage types are never nested
//       inside of non-storage types.
//       E.g., we don't expect to have a storage fields like these:
//         field_a: (u8, u8, StorageMap<...>) = (1, 2, StorageMap {}),
//         field_b: SomeNonStorageTypeStruct<StorageMap<...>> = SomeNonStorageTypeStruct { field: StorageMap {} },

fn review_existing_usages_of_storage_and_str_array(
    program_info: &ProgramInfo,
) -> Result<Vec<Occurrence>> {
    ProgramVisitor::visit_program(program_info, DryRun::Yes, &mut Visitor::default())
}

#[derive(Default)]
struct Visitor {}

impl TreesVisitor<Occurrence> for Visitor {
    fn visit_storage_field_decl(
        &mut self,
        ctx: &VisitingContext,
        lexed_storage_field: &StorageField,
        ty_storage_field: Option<&TyStorageField>,
        output: &mut Vec<Occurrence>,
    ) -> Result<InvalidateTypedElement> {
        if let Some(ty_field_type) = ty_storage_field.map(|x| x.type_argument.type_id) {
            let flag_this_field = ty_field_type.extract_any_including_self(
                ctx.engines,
                &|x| x.is_str_array(),
                vec![],
                0,
            ).iter().any(|(str_array, _)| {
                if let TypeInfo::StringArray(length) = &*ctx.engines.te().get(*str_array) {
                    let Some(length) = length.extract_literal(ctx.engines) else {
                        // storage should always have literal lengths, in any case
                        // it is safer to flag it
                        return true
                    };

                    !length.is_multiple_of(8)
                } else {
                    // should be unreachable
                    false
                }
            });

            if flag_this_field {
                output.push(Occurrence::new(
                    lexed_storage_field.name.span(),
                    "Review this field".to_string(),
                ));
                return Ok(InvalidateTypedElement::Yes);
            }
        } else {
            todo!()
        }

        Ok(InvalidateTypedElement::No)
    }
}
