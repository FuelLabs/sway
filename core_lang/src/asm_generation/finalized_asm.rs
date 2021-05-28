use super::{DataSection, InstructionSet};
use crate::error::*;
use either::Either;
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
    // The below invariant is introduced to word-align the data section.
    // A noop is inserted in ASM generation if there is an odd number of ops.
    assert_eq!(program_section.ops.len() % 2, 0);
    let offset_to_data_section = (program_section.ops.len() * 4) as u64;

    // each op is four bytes, so the length of the buf is then number of ops times four.
    let mut buf = vec![0; program_section.ops.len() * 4];

    for (ix, op) in program_section.ops.iter().enumerate() {
        let op = op.to_fuel_asm(offset_to_data_section, data_section);
        match op {
            Either::Right(data) => {
                for i in 0..data.len() {
                    buf[ix + i] = data[i];
                }
            }
            Either::Left(mut op) => {
                op.write(&buf[ix * 4..])
                    .expect("Failed to write to in-memory buffer.");
            }
        }
    }

    let mut data_section = data_section.serialize_to_bytes();

    buf.append(&mut data_section);

    ok(buf, vec![], vec![])
}
