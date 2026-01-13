#![allow(dead_code)]

use super::{MigrationStep, MigrationStepKind};
use crate::{
    migrations::{DryRun, Occurrence, ProgramInfo},
    visiting::{
        InvalidateTypedElement, ProgramVisitor, TreesVisitor, VisitingContext,
    },
};
use anyhow::{Ok, Result};
use sway_ast::StorageField;
use sway_core::{TypeInfo, 
    language::
        ty::TyStorageField}
    
;
use sway_types::Spanned;

pub(super) const REVIEW_EXISTING_USAGES_OF_STORAGE_STR_ARRAY: MigrationStep = MigrationStep {
    title: "Review storage of string arrays",
    duration: 10,
    kind: MigrationStepKind::Instruction(review_existing_usages_of_storage_and_str_array),
    help: &[
        "Runtime layout of string arrays is changing. Currently they are padded until reach",
        "size multiple of 8. This was done to improve their usage on encoding v0.",
        "",
        "Given that encoding v0 is reach its end of life, and encoding v1 will be the unique option",
        "string array layout can be fixed and improve its performance.",
        "",
        "This difference is layout has impact on storage, because now types will fall into different slots.",
        "To avoid issues one can:",
        "  - change from str[N] to [u8; M] where M is next multiple of 8 after N,",
        "  - migrate all types using <SOMETHING>",
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
struct Visitor { }

impl TreesVisitor<Occurrence> for Visitor {
    fn visit_storage_field_decl(
        &mut self,
        ctx: &VisitingContext,
        lexed_storage_field: &StorageField,
        ty_storage_field: Option<&TyStorageField>,
        output: &mut Vec<Occurrence>,
    ) -> Result<InvalidateTypedElement> {
        if let Some(ty_field_type) = ty_storage_field.map(|x| x.type_argument.type_id) {
            let str_arrays = ty_field_type.extract_any_including_self(ctx.engines, &|x| x.is_str_array(), vec![], 0);

            for (str_array, _) in str_arrays {
                match &*ctx.engines.te().get(str_array) {
                    TypeInfo::StringArray(length) => {
                        let Some(length) = length.extract_literal(ctx.engines) else {
                            todo!()
                        };
                        if !length.is_multiple_of(8) {
                            output.push(Occurrence::new(
                                lexed_storage_field.name.span(),
                                "Review this field".to_string(),
                            ));
                            return Ok(InvalidateTypedElement::Yes);
                        }
                    },
                    _ => {

                    }
                }
            }
        } else {
            todo!()
        }

        Ok(InvalidateTypedElement::No)
    }
}

