use super::{DataSection, InstructionSet};
use crate::asm_lang::virtual_ops::VirtualImmediate12;
use crate::error::*;
use std::io::Write;
/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
pub enum FinalizedAsm<'sc> {
    ContractAbi,
    ScriptMain {
        data_section: DataSection<'sc>,
        program_section: InstructionSet<'sc>,
    },
    PredicateMain {
        data_section: DataSection<'sc>,
        program_section: InstructionSet<'sc>,
    },
    // Libraries do not generate any asm.
    Library,
}

impl<'sc> FinalizedAsm<'sc> {
    pub(crate) fn to_bytecode(&self) -> CompileResult<'sc, Vec<u8>> {
        use FinalizedAsm::*;
        match self {
            ContractAbi | Library => ok(vec![], vec![], vec![]),
            ScriptMain {
                program_section,
                data_section,
            } => to_bytecode(program_section, data_section),
            PredicateMain {
                program_section,
                data_section,
            } => to_bytecode(program_section, data_section),
        }
    }
}

fn to_bytecode<'sc>(
    program_section: &InstructionSet<'sc>,
    data_section: &DataSection<'sc>,
) -> CompileResult<'sc, Vec<u8>> {
    let offset_to_data_section = program_section.ops.len() as u64;
    // TODO anything else besides this -- we should not have a 2^12 limit on instructions
    if offset_to_data_section > crate::asm_generation::compiler_constants::TWELVE_BITS {
        return err(
            vec![],
            vec![CompileError::TooManyInstructions {
                span: pest::Span::new("  ", 0, 0).unwrap(),
            }],
        );
    }

    let offset_to_data_section = VirtualImmediate12::new_unchecked(
        offset_to_data_section,
        "this was manually checked with [CompileError::TooManyInstructions] above. ",
    );

    // each op is four bytes, so the length of the buf is then number of ops times four.
    let mut buf = vec![0; program_section.ops.len() * 4];

    for (ix, op) in program_section.ops.iter().enumerate() {
        let mut op = op.to_fuel_asm(&offset_to_data_section, data_section);
        op.write(&buf[ix * 4..])
            .expect("Failed to write to in-memory buffer.");
    }

    let mut data_section = data_section.serialize_to_bytes();

    buf.append(&mut data_section);

    ok(buf, vec![], vec![])
}
