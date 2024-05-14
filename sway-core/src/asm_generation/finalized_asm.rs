use super::instruction_set::InstructionSet;
use super::ToMidenBytecode;
use super::{
    fuel::{checks, data_section::DataSection},
    ProgramABI, ProgramKind,
};
use crate::asm_lang::allocated_ops::{AllocatedOp, AllocatedOpcode};
use crate::decl_engine::DeclRefFunction;
use crate::source_map::SourceMap;
use crate::BuildConfig;

use etk_asm::asm::Assembler;
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::span::Span;
use sway_types::SourceEngine;

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
        handler: &Handler,
        source_map: &mut SourceMap,
        source_engine: &SourceEngine,
        build_config: &BuildConfig,
    ) -> Result<CompiledBytecode, ErrorEmitted> {
        match &self.program_section {
            InstructionSet::Fuel { ops } => Ok(to_bytecode_mut(
                ops,
                &mut self.data_section,
                source_map,
                source_engine,
                build_config,
            )),
            InstructionSet::Evm { ops } => {
                let mut assembler = Assembler::new();
                if let Err(e) = assembler.push_all(ops.clone()) {
                    Err(handler.emit_err(CompileError::InternalOwned(e.to_string(), Span::dummy())))
                } else {
                    Ok(CompiledBytecode {
                        bytecode: assembler.take(),
                        config_const_offsets: BTreeMap::new(),
                    })
                }
            }
            InstructionSet::MidenVM { ops } => Ok(CompiledBytecode {
                bytecode: ops.to_bytecode().into(),
                config_const_offsets: Default::default(),
            }),
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
    ops: &[AllocatedOp],
    data_section: &mut DataSection,
    source_map: &mut SourceMap,
    source_engine: &SourceEngine,
    build_config: &BuildConfig,
) -> CompiledBytecode {
    fn op_size_in_bytes(data_section: &DataSection, item: &AllocatedOp) -> u64 {
        match &item.opcode {
            AllocatedOpcode::LoadDataId(_reg, data_label)
                if !data_section
                    .has_copy_type(data_label)
                    .expect("data label references non existent data -- internal error") =>
            {
                8
            }
            AllocatedOpcode::DataSectionOffsetPlaceholder => 8,
            AllocatedOpcode::BLOB(count) => count.value as u64 * 4,
            AllocatedOpcode::CFEI(i) | AllocatedOpcode::CFSI(i) if i.value == 0 => 0,
            _ => 4,
        }
    }

    // Some instructions may be omitted or expanded into multiple instructions, so we compute,
    // using `op_size_in_bytes`, exactly how many ops will be generated to calculate the offset.
    let mut offset_to_data_section_in_bytes = ops
        .iter()
        .fold(0, |acc, item| acc + op_size_in_bytes(data_section, item));

    // A noop is inserted in ASM generation if required, to word-align the data section.
    let mut ops_padded = Vec::new();
    let ops = if offset_to_data_section_in_bytes & 7 == 0 {
        ops
    } else {
        ops_padded.reserve(ops.len() + 1);
        ops_padded.extend(ops.iter().cloned());
        ops_padded.push(AllocatedOp {
            opcode: AllocatedOpcode::NOOP,
            comment: "word-alignment of data section".into(),
            owning_span: None,
        });
        offset_to_data_section_in_bytes += 4;
        &ops_padded
    };

    let mut buf = Vec::with_capacity(offset_to_data_section_in_bytes as usize);

    if build_config.print_bytecode {
        println!(";; --- START OF TARGET BYTECODE ---\n");
    }

    let mut half_word_ix = 0;
    let mut offset_from_instr_start = 0;
    for op in ops.iter() {
        let span = op.owning_span.clone();
        let fuel_op = op.to_fuel_asm(
            offset_to_data_section_in_bytes,
            offset_from_instr_start,
            data_section,
        );
        offset_from_instr_start += op_size_in_bytes(data_section, op);

        match fuel_op {
            Either::Right(data) => {
                if build_config.print_bytecode {
                    println!("{:?}", data);
                }
                // Static assert to ensure that we're only dealing with DataSectionOffsetPlaceholder,
                // a one-word (8 bytes) data within the code. No other uses are known.
                let _: [u8; 8] = data;
                buf.extend(data.iter().cloned());
                half_word_ix += 2;
            }
            Either::Left(ops) => {
                for op in ops {
                    if build_config.print_bytecode {
                        println!("{:?}", op);
                    }
                    if let Some(span) = &span {
                        source_map.insert(source_engine, half_word_ix, span);
                    }
                    buf.extend(op.to_bytes().iter());
                    half_word_ix += 1;
                }
            }
        }
    }
    if build_config.print_bytecode {
        println!("{}", data_section);
        println!(";; --- END OF TARGET BYTECODE ---\n");
    }

    assert_eq!(half_word_ix * 4, offset_to_data_section_in_bytes as usize);
    assert_eq!(buf.len(), offset_to_data_section_in_bytes as usize);

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

    CompiledBytecode {
        bytecode: buf,
        config_const_offsets: config_offsets,
    }
}

/// Checks for disallowed opcodes in non-contract code.
/// i.e., if this is a script or predicate, we can't use certain contract opcodes.
/// See https://github.com/FuelLabs/sway/issues/350 for details.
pub fn check_invalid_opcodes(handler: &Handler, asm: &FinalizedAsm) -> Result<(), ErrorEmitted> {
    match &asm.program_section {
        InstructionSet::Fuel { ops } => match asm.program_kind {
            ProgramKind::Contract | ProgramKind::Library => Ok(()),
            ProgramKind::Script => checks::check_script_opcodes(handler, &ops[..]),
            ProgramKind::Predicate => checks::check_predicate_opcodes(handler, &ops[..]),
        },
        InstructionSet::Evm { ops: _ } => Ok(()),
        InstructionSet::MidenVM { ops: _ } => Ok(()),
    }
}
