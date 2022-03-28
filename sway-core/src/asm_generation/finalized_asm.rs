use super::{DataSection, InstructionSet};
use crate::asm_lang::allocated_ops::AllocatedOpcode;
use crate::error::*;
use crate::source_map::SourceMap;

use sway_types::span::Span;

use either::Either;
use std::io::Read;

/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
#[derive(Clone)]
pub enum FinalizedAsm {
    ContractAbi {
        data_section: DataSection,
        program_section: InstructionSet,
    },
    ScriptMain {
        data_section: DataSection,
        program_section: InstructionSet,
    },
    PredicateMain {
        data_section: DataSection,
        program_section: InstructionSet,
    },
    // Libraries do not generate any asm.
    Library,
}
impl FinalizedAsm {
    pub(crate) fn to_bytecode_mut(&mut self, source_map: &mut SourceMap) -> CompileResult<Vec<u8>> {
        use FinalizedAsm::*;
        match self {
            ContractAbi {
                program_section,
                ref mut data_section,
            } => to_bytecode_mut(program_section, data_section, source_map),
            // libraries are not compiled to asm
            Library => ok(vec![], vec![], vec![]),
            ScriptMain {
                program_section,
                ref mut data_section,
            } => to_bytecode_mut(program_section, data_section, source_map),
            PredicateMain {
                program_section,
                ref mut data_section,
            } => to_bytecode_mut(program_section, data_section, source_map),
        }
    }
}

fn to_bytecode_mut(
    program_section: &InstructionSet,
    data_section: &mut DataSection,
    source_map: &mut SourceMap,
) -> CompileResult<Vec<u8>> {
    let mut errors = vec![];
    if program_section.ops.len() & 1 != 0 {
        println!("ops len: {}", program_section.ops.len());
        errors.push(CompileError::Internal(
            "Non-word-aligned (odd-number) ops generated. This is an invariant violation.",
            Span::new(" ".into(), 0, 0, None).unwrap(),
        ));
        return err(vec![], errors);
    }
    // The below invariant is introduced to word-align the data section.
    // A noop is inserted in ASM generation if there is an odd number of ops.
    assert_eq!(program_section.ops.len() & 1, 0);
    // this points at the byte (*4*8) address immediately following (+1) the last instruction
    // Some LWs are expanded into two ops to allow for data larger than one word, so we calculate
    // exactly how many ops will be generated to calculate the offset.
    let offset_to_data_section_in_bytes =
        program_section
            .ops
            .iter()
            .fold(0, |acc, item| match &item.opcode {
                AllocatedOpcode::LWDataId(_reg, data_label)
                    if data_section
                        .type_of_data(data_label)
                        .expect("data label references non existent data -- internal error")
                        .stack_size_of()
                        > 1 =>
                {
                    acc + 8
                }
                _ => acc + 4,
            })
            + 4;

    // each op is four bytes, so the length of the buf is the number of ops times four.
    let mut buf = vec![0; (program_section.ops.len() * 4) + 4];

    let mut half_word_ix = 0;
    for op in program_section.ops.iter() {
        let span = op.owning_span.clone();
        let op = op.to_fuel_asm(offset_to_data_section_in_bytes, data_section);
        match op {
            Either::Right(data) => {
                for i in 0..data.len() {
                    buf[(half_word_ix * 4) + i] = data[i];
                }
                half_word_ix += 2;
            }
            Either::Left(ops) => {
                if ops.len() > 1 {
                    buf.resize(buf.len() + ((ops.len() - 1) * 4), 0);
                }
                for mut op in ops {
                    if let Some(span) = &span {
                        source_map.insert(half_word_ix, span);
                    }
                    op.read_exact(&mut buf[half_word_ix * 4..])
                        .expect("Failed to write to in-memory buffer.");
                    half_word_ix += 1;
                }
            }
        }
    }

    let mut data_section = data_section.serialize_to_bytes();

    buf.append(&mut data_section);

    ok(buf, vec![], errors)
}
