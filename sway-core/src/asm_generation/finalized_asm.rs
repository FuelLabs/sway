use super::instruction_set::InstructionSet;
use super::{
    fuel::{checks, data_section::DataSection},
    ProgramABI, ProgramKind,
};
use crate::asm_generation::fuel::data_section::{Datum, Entry, EntryName};
use crate::asm_lang::allocated_ops::{AllocatedOp, AllocatedInstruction, FuelAsmData};
use crate::decl_engine::DeclRefFunction;
use crate::source_map::SourceMap;
use crate::BuildConfig;

use etk_asm::asm::Assembler;
use fuel_vm::fuel_asm::{Imm06, Imm12, Imm18, Imm24, Instruction, RegId};
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::span::Span;
use sway_types::SourceEngine;

use std::{collections::BTreeMap, fmt};

/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
#[derive(Clone, serde::Serialize)]
pub struct AsmInformation {
    pub bytecode_size: u64,
    pub data_section: DataSectionInformation,
}

#[derive(Default, Clone, Debug, serde::Serialize)]
pub struct DataSectionInformation {
    /// The total size of the data section in bytes
    pub size: u64,
    /// The used size of the data section in bytes
    pub used: u64,
    /// The data to be put in the data section of the asm
    pub value_pairs: Vec<Entry>,
}

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
    pub named_data_section_entries_offsets: BTreeMap<String, u64>,
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
                        named_data_section_entries_offsets: BTreeMap::new(),
                    })
                }
            }
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
            AllocatedInstruction::LoadDataId(_reg, data_label)
                if !data_section
                    .has_copy_type(data_label)
                    .expect("data label references non existent data -- internal error") =>
            {
                8
            }
            AllocatedInstruction::AddrDataId(_, id)
                if data_section.data_id_to_offset(id) > usize::from(Imm12::MAX.to_u16()) =>
            {
                8
            }
            AllocatedInstruction::ConfigurablesOffsetPlaceholder => 8,
            AllocatedInstruction::DataSectionOffsetPlaceholder => 8,
            AllocatedInstruction::BLOB(count) => count.value() as u64 * 4,
            AllocatedInstruction::CFEI(i) | AllocatedInstruction::CFSI(i) if i.value() == 0 => 0,
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
            opcode: AllocatedInstruction::NOOP,
            comment: "word-alignment of data section".into(),
            owning_span: None,
        });
        offset_to_data_section_in_bytes += 4;
        &ops_padded
    };

    let mut offset_from_instr_start = 0;
    for op in ops.iter() {
        match &op.opcode {
            AllocatedInstruction::LoadDataId(_reg, data_label)
                if !data_section
                    .has_copy_type(data_label)
                    .expect("data label references non existent data -- internal error") =>
            {
                // For non-copy type loads, pre-insert pointers into the data_section so that
                // from this point on, the data_section remains immutable. This is necessary
                // so that when we take addresses of configurables, that address doesn't change
                // later on if a non-configurable is added to the data-section.
                let offset_bytes = data_section.data_id_to_offset(data_label) as u64;
                // The -4 is because $pc is added in the *next* instruction.
                let pointer_offset_from_current_instr =
                    offset_to_data_section_in_bytes - offset_from_instr_start + offset_bytes - 4;
                data_section.append_pointer(pointer_offset_from_current_instr);
            }
            _ => (),
        }
        offset_from_instr_start += op_size_in_bytes(data_section, op);
    }

    let mut bytecode = Vec::with_capacity(offset_to_data_section_in_bytes as usize);

    if build_config.print_bytecode {
        println!(";; --- START OF TARGET BYTECODE ---\n");
    }

    let mut last_span = None;
    let mut indentation = if build_config.print_bytecode_spans {
        4
    } else {
        0
    };

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
            FuelAsmData::DatasectionOffset(data) => {
                if build_config.print_bytecode {
                    print!("{}{:#010x} ", " ".repeat(indentation), bytecode.len());
                    println!(
                        "                                                ;; {:?}",
                        data
                    );
                }

                // Static assert to ensure that we're only dealing with DataSectionOffsetPlaceholder,
                // a one-word (8 bytes) data within the code. No other uses are known.
                let _: [u8; 8] = data;

                bytecode.extend(data.iter().cloned());
                half_word_ix += 2;
            }
            FuelAsmData::ConfigurablesOffset(data) => {
                if build_config.print_bytecode {
                    print!("{}{:#010x} ", " ".repeat(indentation), bytecode.len());
                    println!(
                        "                                                ;; {:?}",
                        data
                    );
                }

                // Static assert to ensure that we're only dealing with ConfigurablesOffsetPlaceholder,
                // a 1-word (8 bytes) data within the code. No other uses are known.
                let _: [u8; 8] = data;

                bytecode.extend(data.iter().cloned());
                half_word_ix += 2;
            }
            FuelAsmData::Instructions(instructions) => {
                for instruction in instructions {
                    // Print original source span only once
                    if build_config.print_bytecode_spans {
                        last_span = match (last_span, &span) {
                            (None, Some(span)) => {
                                indentation = 4;
                                let line_col = span.start_line_col_one_index();
                                println!(
                                    "{} @ {}:{}:{}",
                                    span.as_str(),
                                    span.source_id()
                                        .map(|source_id| source_engine.get_path(source_id))
                                        .map(|x| x.display().to_string())
                                        .unwrap_or("<autogenerated>".to_string()),
                                    line_col.line,
                                    line_col.col
                                );
                                Some(span.clone())
                            }
                            (Some(last), Some(span)) if last != *span => {
                                indentation = 4;
                                let line_col = span.start_line_col_one_index();
                                println!(
                                    "{} @ {}:{}:{}",
                                    span.as_str(),
                                    span.source_id()
                                        .map(|source_id| source_engine.get_path(source_id))
                                        .map(|x| x.display().to_string())
                                        .unwrap_or("<autogenerated>".to_string()),
                                    line_col.line,
                                    line_col.col
                                );
                                Some(span.clone())
                            }
                            (last, _) => last,
                        };
                    }

                    if build_config.print_bytecode {
                        print!("{}{:#010x} ", " ".repeat(indentation), bytecode.len());
                        print_instruction(&instruction);
                    }

                    if let Some(span) = &span {
                        source_map.insert(source_engine, half_word_ix, span);
                    }

                    let bytes = instruction.to_bytes();

                    if build_config.print_bytecode {
                        println!(";; {bytes:?}")
                    }

                    bytecode.extend(bytes.iter());
                    half_word_ix += 1;
                }
            }
        }
    }

    if build_config.print_bytecode {
        println!(".data_section:");

        let offset = bytecode.len();

        fn print_entry(indentation: usize, offset: usize, pair: &Entry) {
            print!("{}{:#010x} ", " ".repeat(indentation), offset);

            match &pair.value {
                Datum::Byte(w) => println!(".byte i{w}, as hex {w:02X}"),
                Datum::Word(w) => {
                    println!(".word i{w}, as hex be bytes ({:02X?})", w.to_be_bytes())
                }
                Datum::ByteArray(bs) => {
                    print!(".bytes as hex ({bs:02X?}), len i{}, as ascii \"", bs.len());

                    for b in bs {
                        print!(
                            "{}",
                            if *b == b' ' || b.is_ascii_graphic() {
                                *b as char
                            } else {
                                '.'
                            }
                        );
                    }
                    println!("\"");
                }
                Datum::Slice(bs) => {
                    print!(".slice as hex ({bs:02X?}), len i{}, as ascii \"", bs.len());

                    for b in bs {
                        print!(
                            "{}",
                            if *b == b' ' || b.is_ascii_graphic() {
                                *b as char
                            } else {
                                '.'
                            }
                        );
                    }
                    println!("\"");
                }
                Datum::Collection(els) => {
                    println!(".collection");
                    for e in els {
                        print_entry(indentation + 1, offset, e);
                    }
                }
            };
        }

        for (i, entry) in data_section.iter_all_entries().enumerate() {
            let entry_offset = data_section.absolute_idx_to_offset(i);
            print_entry(indentation, offset + entry_offset, &entry);
        }

        println!(";; --- END OF TARGET BYTECODE ---\n");
    }

    assert_eq!(half_word_ix * 4, offset_to_data_section_in_bytes as usize);
    assert_eq!(bytecode.len(), offset_to_data_section_in_bytes as usize);

    let num_nonconfigurables = data_section.non_configurables.len();
    let named_data_section_entries_offsets = data_section
        .configurables
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            let EntryName::Configurable(name) = &entry.name else {
                panic!("Non-configurable in configurables part of datasection");
            };
            (
                name.clone(),
                offset_to_data_section_in_bytes
                    + data_section.absolute_idx_to_offset(id + num_nonconfigurables) as u64,
            )
        })
        .collect::<BTreeMap<String, u64>>();

    let mut data_section = data_section.serialize_to_bytes();
    bytecode.append(&mut data_section);

    CompiledBytecode {
        bytecode,
        named_data_section_entries_offsets,
    }
}

// Code to pretty print bytecode
fn print_reg(r: RegId) -> String {
    match r {
        RegId::BAL => "$bal".to_string(),
        RegId::CGAS => "$cgas".to_string(),
        RegId::ERR => "$err".to_string(),
        RegId::FLAG => "$flag".to_string(),
        RegId::FP => "$fp".to_string(),
        RegId::GGAS => "$ggas".to_string(),
        RegId::HP => "$hp".to_string(),
        RegId::IS => "$is".to_string(),
        RegId::OF => "$of".to_string(),
        RegId::ONE => "$one".to_string(),
        RegId::PC => "$pc".to_string(),
        RegId::RET => "$ret".to_string(),
        RegId::RETL => "$retl".to_string(),
        RegId::SP => "$sp".to_string(),
        RegId::SSP => "$ssp".to_string(),
        RegId::WRITABLE => "$writable".to_string(),
        RegId::ZERO => "$zero".to_string(),
        _ => format!("R{:?}", r.to_u8()),
    }
}

trait Args {
    fn print(&self) -> String;
}

impl Args for RegId {
    fn print(&self) -> String {
        print_reg(*self)
    }
}
impl Args for Imm06 {
    fn print(&self) -> String {
        format!("{:#x}", self.to_u8())
    }
}
impl Args for Imm12 {
    fn print(&self) -> String {
        format!("{:#x}", self.to_u16())
    }
}
impl Args for Imm18 {
    fn print(&self) -> String {
        format!("{:#x}", self.to_u32())
    }
}
impl Args for Imm24 {
    fn print(&self) -> String {
        format!("{:#x}", self.to_u32())
    }
}
impl Args for () {
    fn print(&self) -> String {
        String::new()
    }
}
impl<A: Args> Args for (A,) {
    fn print(&self) -> String {
        self.0.print()
    }
}
impl<A: Args, B: Args> Args for (A, B) {
    fn print(&self) -> String {
        format!("{} {}", self.0.print(), self.1.print())
    }
}
impl<A: Args, B: Args, C: Args> Args for (A, B, C) {
    fn print(&self) -> String {
        format!("{} {} {}", self.0.print(), self.1.print(), self.2.print())
    }
}
impl<A: Args, B: Args, C: Args, D: Args> Args for (A, B, C, D) {
    fn print(&self) -> String {
        format!(
            "{} {} {} {}",
            self.0.print(),
            self.1.print(),
            self.2.print(),
            self.3.print()
        )
    }
}

fn f(name: &str, args: impl Args) {
    let mut line = format!("{name} {}", args.print());
    let s = " ".repeat(48 - line.len());
    line.push_str(&s);
    print!("{line}")
}

fn print_instruction(op: &Instruction) {
    match op {
        Instruction::ADD(x) => f("ADD", x.unpack()),
        Instruction::AND(x) => f("AND", x.unpack()),
        Instruction::DIV(x) => f("DIV", x.unpack()),
        Instruction::EQ(x) => f("EQ", x.unpack()),
        Instruction::EXP(x) => f("EXP", x.unpack()),
        Instruction::GT(x) => f("GT", x.unpack()),
        Instruction::LT(x) => f("LT", x.unpack()),
        Instruction::MLOG(x) => f("MLOG", x.unpack()),
        Instruction::MROO(x) => f("MROO", x.unpack()),
        Instruction::MOD(x) => f("MOD", x.unpack()),
        Instruction::MOVE(x) => f("MOVE", x.unpack()),
        Instruction::MUL(x) => f("MUL", x.unpack()),
        Instruction::NOT(x) => f("NOT", x.unpack()),
        Instruction::OR(x) => f("OR", x.unpack()),
        Instruction::SLL(x) => f("SLL", x.unpack()),
        Instruction::SRL(x) => f("SRL", x.unpack()),
        Instruction::SUB(x) => f("SUB", x.unpack()),
        Instruction::XOR(x) => f("XOR", x.unpack()),
        Instruction::MLDV(x) => f("MLDV", x.unpack()),
        Instruction::RET(x) => f("RET", x.unpack()),
        Instruction::RETD(x) => f("RETD", x.unpack()),
        Instruction::ALOC(x) => f("ALOC", x.unpack()),
        Instruction::MCL(x) => f("MCL", x.unpack()),
        Instruction::MCP(x) => f("MCP", x.unpack()),
        Instruction::MEQ(x) => f("MEQ", x.unpack()),
        Instruction::BHSH(x) => f("BHSH", x.unpack()),
        Instruction::BHEI(x) => f("BHEI", x.unpack()),
        Instruction::BURN(x) => f("BURN", x.unpack()),
        Instruction::CALL(x) => f("CALL", x.unpack()),
        Instruction::CCP(x) => f("CCP", x.unpack()),
        Instruction::CROO(x) => f("CROO", x.unpack()),
        Instruction::CSIZ(x) => f("CSIZ", x.unpack()),
        Instruction::CB(x) => f("CB", x.unpack()),
        Instruction::LDC(x) => f("LDC", x.unpack()),
        Instruction::LOG(x) => f("LOG", x.unpack()),
        Instruction::LOGD(x) => f("LOGD", x.unpack()),
        Instruction::MINT(x) => f("MINT", x.unpack()),
        Instruction::RVRT(x) => f("RVRT", x.unpack()),
        Instruction::SCWQ(x) => f("SCWQ", x.unpack()),
        Instruction::SRW(x) => f("SRW", x.unpack()),
        Instruction::SRWQ(x) => f("SRWQ", x.unpack()),
        Instruction::SWW(x) => f("SWW", x.unpack()),
        Instruction::SWWQ(x) => f("SWWQ", x.unpack()),
        Instruction::TR(x) => f("TR", x.unpack()),
        Instruction::TRO(x) => f("TRO", x.unpack()),
        Instruction::ECK1(x) => f("ECK1", x.unpack()),
        Instruction::ECR1(x) => f("ECR1", x.unpack()),
        Instruction::ED19(x) => f("ED19", x.unpack()),
        Instruction::K256(x) => f("K256", x.unpack()),
        Instruction::S256(x) => f("S256", x.unpack()),
        Instruction::TIME(x) => f("TIME", x.unpack()),
        Instruction::NOOP(_) => f("NOOP", ()),
        Instruction::FLAG(x) => f("FLAG", x.unpack()),
        Instruction::BAL(x) => f("BAL", x.unpack()),
        Instruction::JMP(x) => f("JMP", x.unpack()),
        Instruction::JNE(x) => f("JNE", x.unpack()),
        Instruction::SMO(x) => f("SMO", x.unpack()),
        Instruction::ADDI(x) => f("ADDI", x.unpack()),
        Instruction::ANDI(x) => f("ANDI", x.unpack()),
        Instruction::DIVI(x) => f("DIVI", x.unpack()),
        Instruction::EXPI(x) => f("EXPI", x.unpack()),
        Instruction::MODI(x) => f("MODI", x.unpack()),
        Instruction::MULI(x) => f("MULI", x.unpack()),
        Instruction::ORI(x) => f("ORI", x.unpack()),
        Instruction::SLLI(x) => f("SLLI", x.unpack()),
        Instruction::SRLI(x) => f("SRLI", x.unpack()),
        Instruction::SUBI(x) => f("SUBI", x.unpack()),
        Instruction::XORI(x) => f("XORI", x.unpack()),
        Instruction::JNEI(x) => f("JNEI", x.unpack()),
        Instruction::LB(x) => f("LB", x.unpack()),
        Instruction::LW(x) => f("LW", x.unpack()),
        Instruction::SB(x) => f("SB", x.unpack()),
        Instruction::SW(x) => f("SW", x.unpack()),
        Instruction::MCPI(x) => f("MCPI", x.unpack()),
        Instruction::GTF(x) => f("GTF", x.unpack()),
        Instruction::MCLI(x) => f("MCLI", x.unpack()),
        Instruction::GM(x) => f("GM", x.unpack()),
        Instruction::MOVI(x) => f("MOVI", x.unpack()),
        Instruction::JNZI(x) => f("JNZI", x.unpack()),
        Instruction::JMPF(x) => f("JMPF", x.unpack()),
        Instruction::JMPB(x) => f("JMPB", x.unpack()),
        Instruction::JNZF(x) => f("JNZF", x.unpack()),
        Instruction::JNZB(x) => f("JNZB", x.unpack()),
        Instruction::JNEF(x) => f("JNEF", x.unpack()),
        Instruction::JNEB(x) => f("JNEB", x.unpack()),
        Instruction::JI(x) => f("JI", x.unpack()),
        Instruction::CFEI(x) => f("CFEI", x.unpack()),
        Instruction::CFSI(x) => f("CFSI", x.unpack()),
        Instruction::CFE(x) => f("CFE", x.unpack()),
        Instruction::CFS(x) => f("CFS", x.unpack()),
        Instruction::PSHL(x) => f("PSHL", x.unpack()),
        Instruction::PSHH(x) => f("PSHH", x.unpack()),
        Instruction::POPL(x) => f("POPL", x.unpack()),
        Instruction::POPH(x) => f("POPH", x.unpack()),
        Instruction::WDCM(x) => f("WDCM", x.unpack()),
        Instruction::WQCM(x) => f("WQCM", x.unpack()),
        Instruction::WDOP(x) => f("WDOP", x.unpack()),
        Instruction::WQOP(x) => f("WQOP", x.unpack()),
        Instruction::WDML(x) => f("WDML", x.unpack()),
        Instruction::WQML(x) => f("WQML", x.unpack()),
        Instruction::WDDV(x) => f("WDDV", x.unpack()),
        Instruction::WQDV(x) => f("WQDV", x.unpack()),
        Instruction::WDMD(x) => f("WDMD", x.unpack()),
        Instruction::WQMD(x) => f("WQMD", x.unpack()),
        Instruction::WDAM(x) => f("WDAM", x.unpack()),
        Instruction::WQAM(x) => f("WQAM", x.unpack()),
        Instruction::WDMM(x) => f("WDMM", x.unpack()),
        Instruction::WQMM(x) => f("WQMM", x.unpack()),
        Instruction::ECAL(x) => f("ECAL", x.unpack()),
        Instruction::BSIZ(x) => f("BSIZ", x.unpack()),
        Instruction::BLDD(x) => f("BLDD", x.unpack()),
        Instruction::ECOP(x) => f("ECOP", x.unpack()),
        Instruction::EPAR(x) => f("EPAR", x.unpack()),
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
    }
}
