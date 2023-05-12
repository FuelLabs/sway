use super::instruction_set::InstructionSet;
use super::ToMidenBytecode;
use super::{
    fuel::{checks, data_section::DataSection},
    ProgramABI, ProgramKind,
};
use crate::asm_lang::allocated_ops::{AllocatedOp, AllocatedOpcode};
use crate::decl_engine::DeclRefFunction;
use crate::error::*;
use crate::source_map::SourceMap;

use etk_asm::asm::Assembler;
use sway_error::error::CompileError;
use sway_types::span::Span;

use either::Either;
use std::{collections::BTreeMap, fmt};

/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
#[derive(Clone)]
pub struct FinalizedAsm {
    pub data_section: DataSection,
    pub program_section: InstructionSet,
    pub program_kind: ProgramKind,
    pub entries: Vec<FinalizedEntry>,
    pub abi: Option<ProgramABI>,
}

#[derive(Clone, Debug)]
pub struct FinalizedEntry {
    /// The original entry point function name.
    pub fn_name: String,
    /// The immediate instruction offset at which the entry function begins.
    pub imm: u64,
    /// The function selector (only `Some` for contract ABI methods).
    pub selector: Option<[u8; 4]>,
    /// If this entry is constructed from a test function contains the declaration id for that
    /// function, otherwise contains `None`.
    pub test_decl_ref: Option<DeclRefFunction>,
}

/// The bytecode for a sway program as well as the byte offsets of configuration-time constants in
/// the bytecode.
pub struct CompiledBytecode {
    pub bytecode: Vec<u8>,
    pub config_const_offsets: BTreeMap<String, u64>,
}

impl FinalizedAsm {
    pub(crate) fn to_bytecode_mut(
        &mut self,
        source_map: &mut SourceMap,
    ) -> CompileResult<CompiledBytecode> {
        match &self.program_section {
            InstructionSet::Fuel { ops } => {
                to_bytecode_mut(ops, &mut self.data_section, source_map)
            }
            InstructionSet::Evm { ops } => {
                let mut assembler = Assembler::new();
                if let Err(e) = assembler.push_all(ops.clone()) {
                    err(
                        vec![],
                        vec![CompileError::InternalOwned(e.to_string(), Span::dummy())],
                    )
                } else {
                    ok(
                        CompiledBytecode {
                            bytecode: assembler.take(),
                            config_const_offsets: BTreeMap::new(),
                        },
                        vec![],
                        vec![],
                    )
                }
            }
            InstructionSet::MidenVM { ops } => ok(
                CompiledBytecode {
                    bytecode: ops.to_bytecode().into(),
                    config_const_offsets: Default::default(),
                },
                vec![],
                vec![],
            ),
        }
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
        write!(f, "{}\n{}", self.program_section, self.data_section)
    }
}

fn to_bytecode_mut(
    ops: &Vec<AllocatedOp>,
    data_section: &mut DataSection,
    source_map: &mut SourceMap,
) -> CompileResult<CompiledBytecode> {
    let mut errors = vec![];

    if ops.len() & 1 != 0 {
        tracing::info!("ops len: {}", ops.len());
        errors.push(CompileError::Internal(
            "Non-word-aligned (odd-number) ops generated. This is an invariant violation.",
            Span::new(" ".into(), 0, 0, None).unwrap(),
        ));
        return err(vec![], errors);
    }
    // The below invariant is introduced to word-align the data section.
    // A noop is inserted in ASM generation if there is an odd number of ops.
    assert_eq!(ops.len() & 1, 0);
    // this points at the byte (*4*8) address immediately following (+1) the last instruction
    // Some LWs are expanded into two ops to allow for data larger than one word, so we calculate
    // exactly how many ops will be generated to calculate the offset.
    let offset_to_data_section_in_bytes = ops.iter().fold(0, |acc, item| match &item.opcode {
        AllocatedOpcode::LWDataId(_reg, data_label)
            if !data_section
                .has_copy_type(data_label)
                .expect("data label references non existent data -- internal error") =>
        {
            acc + 8
        }
        AllocatedOpcode::BLOB(count) => acc + count.value as u64 * 4,
        _ => acc + 4,
    }) + 4;

    // each op is four bytes, so the length of the buf is the number of ops times four.
    let mut buf = vec![0; (ops.len() * 4) + 4];

    let mut half_word_ix = 0;
    for op in ops.iter() {
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
                for op in ops {
                    if let Some(span) = &span {
                        source_map.insert(half_word_ix, span);
                    }
                    let read_range_upper_bound =
                        core::cmp::min(half_word_ix * 4 + std::mem::size_of_val(&op), buf.len());
                    buf[half_word_ix * 4..read_range_upper_bound].copy_from_slice(&op.to_bytes());
                    half_word_ix += 1;
                }
            }
        }
    }

    let config_offsets = data_section
        .config_map
        .iter()
        .map(|(name, id)| {
            (
                name.clone(),
                offset_to_data_section_in_bytes + data_section.raw_data_id_to_offset(*id) as u64,
            )
        })
        .collect::<BTreeMap<String, u64>>();

    let mut data_section = data_section.serialize_to_bytes();

    buf.append(&mut data_section);

    ok(
        CompiledBytecode {
            bytecode: buf,
            config_const_offsets: config_offsets,
        },
        vec![],
        errors,
    )
}

/// Checks for disallowed opcodes in non-contract code.
/// i.e., if this is a script or predicate, we can't use certain contract opcodes.
/// See https://github.com/FuelLabs/sway/issues/350 for details.
pub fn check_invalid_opcodes(asm: &FinalizedAsm) -> CompileResult<()> {
    match &asm.program_section {
        InstructionSet::Fuel { ops } => match asm.program_kind {
            ProgramKind::Contract | ProgramKind::Library => ok((), vec![], vec![]),
            ProgramKind::Script => checks::check_script_opcodes(&ops[..]),
            ProgramKind::Predicate => checks::check_predicate_opcodes(&ops[..]),
        },
        InstructionSet::Evm { ops: _ } => ok((), vec![], vec![]),
        InstructionSet::MidenVM { ops: _ } => ok((), vec![], vec![]),
    }
}
