use super::{DataSection, InstructionSet, ProgramKind};
use crate::asm_lang::allocated_ops::AllocatedOpcode;
use crate::error::*;
use crate::source_map::SourceMap;

use sway_error::error::CompileError;
use sway_types::span::Span;

use either::Either;
use std::fmt;
use std::io::Read;

/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
#[derive(Clone)]
pub struct FinalizedAsm {
    pub data_section: DataSection,
    pub program_section: InstructionSet,
    pub program_kind: ProgramKind,
    pub entries: Vec<FinalizedEntry>,
}

#[derive(Clone, Debug)]
pub struct FinalizedEntry {
    /// The original entry point function name.
    pub fn_name: String,
    /// The immediate instruction offset at which the entry function begins.
    pub imm: u64,
}

impl FinalizedAsm {
    pub(crate) fn to_bytecode_mut(&mut self, source_map: &mut SourceMap) -> CompileResult<Vec<u8>> {
        to_bytecode_mut(&self.program_section, &mut self.data_section, source_map)
    }
}

impl fmt::Display for FinalizedAsm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}", self.program_section, self.data_section)
    }
}

fn to_bytecode_mut(
    program_section: &InstructionSet,
    data_section: &mut DataSection,
    source_map: &mut SourceMap,
) -> CompileResult<Vec<u8>> {
    let mut errors = vec![];
    if program_section.ops.len() & 1 != 0 {
        tracing::info!("ops len: {}", program_section.ops.len());
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
                    if !data_section
                        .has_copy_type(data_label)
                        .expect("data label references non existent data -- internal error") =>
                {
                    acc + 8
                }
                AllocatedOpcode::BLOB(count) => acc + count.value as u64 * 4,
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
                    let read_range_upper_bound =
                        core::cmp::min(half_word_ix * 4 + std::mem::size_of_val(&op), buf.len());
                    op.read_exact(&mut buf[half_word_ix * 4..read_range_upper_bound])
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
