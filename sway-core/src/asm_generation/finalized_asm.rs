use super::{InstructionSet, ProgramKind, VirtualDataSection};
use crate::{error::*, source_map::SourceMap};

use sway_error::error::CompileError;
use sway_types::span::Span;

use either::Either;
use std::fmt;
use std::io::Read;

/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
#[derive(Clone)]
pub struct FinalizedAsm {
    pub imm_data_section: VirtualDataSection,
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
    /// The function selector (only `Some` for contract ABI methods).
    pub selector: Option<[u8; 4]>,
}

impl FinalizedAsm {
    pub(crate) fn to_bytecode_mut(&mut self, source_map: &mut SourceMap) -> CompileResult<Vec<u8>> {
        to_bytecode_mut(&self.program_section, &self.imm_data_section, source_map)
    }
}

impl FinalizedEntry {
    /// We assume the entry point is for a test function in the case it is neither an ABI method
    /// (no selector) or it is not "main".
    pub fn is_test(&self) -> bool {
        self.selector.is_none()
            && self.fn_name != sway_types::constants::DEFAULT_ENTRY_POINT_FN_NAME
    }
}

impl fmt::Display for FinalizedAsm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}", self.program_section, self.imm_data_section)
    }
}

fn to_bytecode_mut(
    program_section: &InstructionSet,
    imm_data_section: &VirtualDataSection,
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

    // each op is four bytes, so the length of the buf is the number of ops times four.
    let mut buf = vec![0; (program_section.ops.len() * 4) + 4];

    let mut half_word_ix = 0;
    for op in program_section.ops.iter() {
        let span = op.owning_span.clone();
        let op = op.to_fuel_asm(imm_data_section);
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

    let mut data_section = imm_data_section.serialize_to_bytes();

    buf.append(&mut data_section);

    ok(buf, vec![], errors)
}
