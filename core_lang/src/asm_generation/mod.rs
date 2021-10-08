use std::{collections::HashMap, fmt};

use crate::type_engine::TypeEngine;
use crate::{
    asm_generation::expression::convert_abi_fn_to_asm,
    asm_lang::{
        allocated_ops::{AllocatedOp, AllocatedRegister},
        virtual_ops::{
            ConstantRegister, Label, VirtualImmediate12, VirtualImmediate24, VirtualOp,
            VirtualRegister,
        },
        Op, OrganizationalOp, RealizedOp,
    },
    error::*,
    parse_tree::Literal,
    semantic_analysis::{
        TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration,
        TypedParseTree,
    },
    types::{MaybeResolvedType, ResolvedType},
    BuildConfig, Ident, TypeInfo,
};
use either::Either;

pub(crate) mod compiler_constants;
mod declaration;
mod expression;
mod finalized_asm;
mod register_sequencer;
mod while_loop;

pub(crate) use declaration::*;
pub(crate) use expression::*;
pub use finalized_asm::FinalizedAsm;
pub(crate) use register_sequencer::*;

use crate::type_engine::{Engine, TypeId};
use while_loop::convert_while_loop_to_asm;

// Initially, the bytecode will have a lot of individual registers being used. Each register will
// have a new unique identifier. For example, two separate invocations of `+` will result in 4
// registers being used for arguments and 2 for outputs.
//
// After that, the level 0 bytecode will go through a process where register use is minified,
// producing level 1 bytecode. This process is as such:
//
// 1. Detect the last time a register is read. After that, it can be reused and recycled to fit the
//    needs of the next "level 0 bytecode" register
//
// 2. Detect needless assignments and movements, and substitute registers in.
//    i.e.
//    a = b
//    c = a
//
//    would become
//    c = b
//
//
// After the level 1 bytecode is produced, level 2 bytecode is created by limiting the maximum
// number of registers and inserting bytecode to read from/write to memory where needed. Ideally,
// the algorithm for determining which registers will be written off to memory is based on how
// frequently that register is accessed in a particular section of code. Using this strategy, we
// hope to minimize memory writing.
//
// For each line, the number of times a virtual register is accessed between then and the end of the
// program is its register precedence. A virtual register's precedence is 0 if it is currently in
// "memory", and the above described number if it is not. This prevents over-prioritization of
// registers that have already been written off to memory.
//
/// The [HllAsmSet] contains either a contract ABI and corresponding ASM, a script's main
/// function's ASM, or a predicate's main function's ASM. ASM is never generated for libraries,
/// as that happens when the library itself is imported.
pub enum HllAsmSet<'sc> {
    ContractAbi {
        data_section: DataSection<'sc>,
        program_section: AbstractInstructionSet<'sc>,
    },
    ScriptMain {
        data_section: DataSection<'sc>,
        program_section: AbstractInstructionSet<'sc>,
    },
    PredicateMain {
        data_section: DataSection<'sc>,
        program_section: AbstractInstructionSet<'sc>,
    },
    // Libraries do not generate any asm.
    Library,
}

/// An [AbstractInstructionSet] is a set of instructions that use entirely virtual registers
/// and excessive moves, with the intention of later optimizing it.
#[derive(Clone)]
pub struct AbstractInstructionSet<'sc> {
    ops: Vec<Op<'sc>>,
}

/// "Realized" here refers to labels -- there are no more organizational
/// ops or labels. In this struct, they are all "realized" to offsets.
pub struct RealizedAbstractInstructionSet<'sc> {
    ops: Vec<RealizedOp<'sc>>,
}

impl<'sc> RealizedAbstractInstructionSet<'sc> {
    fn allocate_registers(self) -> InstructionSet<'sc> {
        // Eventually, we will use a cool graph-coloring algorithm.
        // For now, just keep a pool of registers and return
        // registers when they are not read anymore

        // construct a mapping from every op to the registers it uses
        let op_register_mapping = self
            .ops
            .into_iter()
            .map(|op| {
                (
                    op.clone(),
                    op.opcode
                        .registers()
                        .into_iter()
                        .map(|x| x.clone())
                        .collect(),
                )
            })
            .collect::<Vec<_>>();

        // get registers from the pool.
        let mut pool = RegisterPool::init();
        let mut buf = vec![];
        for (ix, (op, _)) in op_register_mapping.iter().enumerate() {
            buf.push(AllocatedOp {
                opcode: op
                    .opcode
                    .allocate_registers(&mut pool, &op_register_mapping, ix),
                comment: op.comment.clone(),
                owning_span: op.owning_span.clone(),
            })
        }
        InstructionSet { ops: buf }
    }
}

/// An [InstructionSet] is produced by allocating registers on an [AbstractInstructionSet].
#[derive(Clone)]
pub struct InstructionSet<'sc> {
    ops: Vec<AllocatedOp<'sc>>,
}

type Data<'sc> = Literal<'sc>;
impl<'sc> AbstractInstructionSet<'sc> {
    /// Removes any jumps that jump to the subsequent line
    fn remove_sequential_jumps(&self) -> AbstractInstructionSet<'sc> {
        let mut buf = vec![];
        for i in 0..self.ops.len() - 1 {
            if let Op {
                opcode: Either::Right(OrganizationalOp::Jump(ref label)),
                ..
            } = self.ops[i]
            {
                if let Op {
                    opcode: Either::Right(OrganizationalOp::Label(ref label2)),
                    ..
                } = self.ops[i + 1]
                {
                    if label == label2 {
                        // this is a jump to the next line
                        // omit these by doing nothing
                        continue;
                    }
                }
            }
            buf.push(self.ops[i].clone());
        }
        // the last item cannot sequentially jump by definition so we add it in here
        self.ops.last().map(|x| buf.push(x.clone()));

        // scan through the jumps and remove any labels that are unused
        // this could of course be N instead of 2N if i did this in the above for loop.
        // However, the sweep for unused labels is inevitable regardless of the above phase
        // so might as well do it here.
        let mut buf2 = vec![];
        for op in &buf {
            match op.opcode {
                Either::Right(OrganizationalOp::Label(ref label)) => {
                    if label_is_used(&buf, label) {
                        buf2.push(op.clone());
                    }
                }
                _ => buf2.push(op.clone()),
            }
        }

        AbstractInstructionSet { ops: buf2 }
    }

    /// Runs two passes -- one to get the instruction offsets of the labels
    /// and one to replace the labels in the organizational ops
    fn realize_labels(
        self,
        data_section: &DataSection,
        namespace: &AsmNamespace,
    ) -> RealizedAbstractInstructionSet<'sc> {
        let mut label_namespace: HashMap<&Label, u64> = Default::default();
        let mut counter = 0;
        for op in &self.ops {
            match op.opcode {
                Either::Right(OrganizationalOp::Label(ref lab)) => {
                    label_namespace.insert(lab, counter);
                }
                // A special case for LWDataId which may be 1 or 2 ops, depending on the source size.
                Either::Left(VirtualOp::LWDataId(_, ref data_id)) => {
                    let type_of_data = data_section.type_of_data(data_id).expect(
                        "Internal miscalculation in data section -- data id did not match up to any actual data",
                    );
                    counter += if type_of_data.stack_size_of(namespace) > 1 {
                        2
                    } else {
                        1
                    };
                }
                // these ops will end up being exactly one op, so the counter goes up one
                Either::Right(OrganizationalOp::Jump(..))
                | Either::Right(OrganizationalOp::JumpIfNotEq(..))
                | Either::Left(_) => {
                    counter += 1;
                }
                Either::Right(OrganizationalOp::Comment) => (),
                Either::Right(OrganizationalOp::DataSectionOffsetPlaceholder) => {
                    // If the placeholder is 32 bits, this is 1. if 64, this should be 2. We use LW
                    // to load the data, which loads a whole word, so for now this is 2.
                    counter += 2
                }
            }
        }

        let mut realized_ops = vec![];
        for Op {
            opcode,
            owning_span,
            comment,
        } in self.ops.clone().into_iter()
        {
            match opcode {
                Either::Left(op) => realized_ops.push(RealizedOp {
                    opcode: op,
                    owning_span,
                    comment,
                }),
                Either::Right(org_op) => match org_op {
                    OrganizationalOp::Jump(ref lab) => {
                        let offset = label_namespace.get(lab).unwrap();
                        let imm = VirtualImmediate24::new_unchecked(
                            *offset,
                            "Programs with more than 2^24 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::JI(imm),
                            owning_span,
                            comment,
                        });
                    }
                    OrganizationalOp::JumpIfNotEq(r1, r2, ref lab) => {
                        let offset = label_namespace.get(lab).unwrap();
                        let imm = VirtualImmediate12::new_unchecked(
                            *offset,
                            "Programs with more than 2^12 labels are unsupported right now",
                        );
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::JNEI(r1, r2, imm),
                            owning_span,
                            comment,
                        });
                    }
                    OrganizationalOp::DataSectionOffsetPlaceholder => {
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::DataSectionOffsetPlaceholder,
                            owning_span: None,
                            comment: String::new(),
                        });
                    }
                    OrganizationalOp::Comment => continue,
                    OrganizationalOp::Label(..) => continue,
                },
            };
        }
        RealizedAbstractInstructionSet { ops: realized_ops }
    }
}

#[derive(Debug)]
struct RegisterAllocationStatus {
    reg: AllocatedRegister,
    in_use: Option<VirtualRegister>,
}
#[derive(Debug)]
pub(crate) struct RegisterPool {
    registers: Vec<RegisterAllocationStatus>,
}

impl RegisterPool {
    fn init() -> Self {
        let register_pool: Vec<RegisterAllocationStatus> = (0
            // - 1 because we reserve the final register for the data_section begin
            ..compiler_constants::NUM_ALLOCATABLE_REGISTERS)
            .map(|x| RegisterAllocationStatus {
                reg: AllocatedRegister::Allocated(x),
                in_use: None,
            })
            .collect();
        Self {
            registers: register_pool,
        }
    }

    /// Checks if any currently used registers are no longer in use, updates the pool,
    /// and grabs an available register.
    pub(crate) fn get_register(
        &mut self,
        virtual_register: &VirtualRegister,
        op_register_mapping: &[(RealizedOp, std::collections::HashSet<VirtualRegister>)],
    ) -> Option<AllocatedRegister> {
        // check if this register has already been allocated for
        if let a @ Some(_) = self.registers.iter().find_map(
            |RegisterAllocationStatus { reg, in_use }| match in_use {
                Some(x) if x == virtual_register => Some(reg),
                _ => None,
            },
        ) {
            return a.cloned();
        }

        // scan to see if any of the old ones are no longer in use
        for RegisterAllocationStatus { in_use, .. } in
            self.registers.iter_mut().filter(|r| r.in_use.is_some())
        {
            if virtual_register_is_never_accessed_again(
                in_use.as_ref().unwrap(),
                op_register_mapping,
            ) {
                *in_use = None;
            }
        }
        // find the next unused register, return it, assign it
        let next_available = self
            .registers
            .iter_mut()
            .find(|RegisterAllocationStatus { in_use, .. }| in_use.is_none());

        match next_available {
            Some(RegisterAllocationStatus { in_use, reg }) => {
                *in_use = Some(virtual_register.clone());
                Some(reg.clone())
            }
            None => None,
        }
    }
}

fn virtual_register_is_never_accessed_again(
    reg: &VirtualRegister,
    ops: &[(RealizedOp, std::collections::HashSet<VirtualRegister>)],
) -> bool {
    !ops.iter().any(|(_, regs)| regs.contains(reg))
}

/// helper function to check if a label is used in a given buffer of ops
fn label_is_used<'sc>(buf: &[Op<'sc>], label: &Label) -> bool {
    buf.iter().any(|Op { ref opcode, .. }| match opcode {
        Either::Right(OrganizationalOp::Jump(ref l)) if label == l => true,
        Either::Right(OrganizationalOp::JumpIfNotEq(_reg0, _reg1, ref l)) if label == l => true,
        _ => false,
    })
}

#[derive(Default, Clone)]
pub struct DataSection<'sc> {
    /// the data to be put in the data section of the asm
    pub value_pairs: Vec<Data<'sc>>,
}

impl<'sc> DataSection<'sc> {
    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn offset_to_id(&self, id: &DataId) -> usize {
        self.value_pairs
            .iter()
            .take(id.0 as usize)
            .map(|x| x.to_bytes().len())
            .sum()
    }

    pub(crate) fn serialize_to_bytes(&self) -> Vec<u8> {
        // not the exact right capacity but serves as a lower bound
        let mut buf = Vec::with_capacity(self.value_pairs.len());
        for val in &self.value_pairs {
            buf.append(&mut val.to_bytes().to_vec());
        }
        buf
    }

    /// Calculates the return type of the data held at a specific [DataId].
    pub(crate) fn type_of_data(&self, id: &DataId) -> Option<ResolvedType<'sc>> {
        self.value_pairs
            .iter()
            .nth(id.0 as usize)
            .map(|x| x.as_type())
    }

    /// When generating code, sometimes a hard-coded data pointer is needed to reference
    /// static values that have a length longer than one word.
    /// This method appends pointers to the end of the data section (thus, not altering the data
    /// offsets of previous data).
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start to the data
    /// in question.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        let pointer_as_data = Literal::new_pointer_literal(pointer_value);
        self.insert_data_value(&pointer_as_data)
    }

    /// Given any data in the form of a [Literal] (using this type mainly because it includes type
    /// information and debug spans), insert it into the data section and return its offset as a
    /// [DataId].
    pub(crate) fn insert_data_value(&mut self, data: &Literal<'sc>) -> DataId {
        // if there is an identical data value, use the same id
        match self.value_pairs.iter().position(|x| x == data) {
            Some(num) => DataId(num as u32),
            None => {
                self.value_pairs.push(data.clone());
                // the index of the data section where the value is stored
                DataId((self.value_pairs.len() - 1) as u32)
            }
        }
    }
}

impl fmt::Display for DataSection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut data_buf = String::new();
        for (ix, data) in self.value_pairs.iter().enumerate() {
            let data_val = match data {
                Literal::U8(num) => format!(".u8 {:#04x}", num),
                Literal::U16(num) => format!(".u16 {:#04x}", num),
                Literal::U32(num) => format!(".u32 {:#04x}", num),
                Literal::U64(num) => format!(".u64 {:#04x}", num),
                Literal::Boolean(b) => format!(".bool {}", if *b { "0x01" } else { "0x00" }),
                Literal::String(st) => format!(".str \"{}\"", st),
                Literal::Byte(b) => format!(".byte {:#08b}", b),
                Literal::B256(b) => format!(
                    ".b256 0x{}",
                    b.into_iter()
                        .map(|x| format!("{:02x}", x))
                        .collect::<Vec<_>>()
                        .join("")
                ),
            };
            let data_label = DataId(ix as u32);
            data_buf.push_str(&format!("{} {}\n", data_label, data_val));
        }

        write!(f, ".data:\n{}", data_buf)
    }
}

impl fmt::Display for HllAsmSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HllAsmSet::ScriptMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            HllAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            HllAsmSet::ContractAbi {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            // Libraries do not directly generate any asm.
            HllAsmSet::Library => write!(f, ""),
        }
    }
}

impl fmt::Display for JumpOptimizedAsmSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            JumpOptimizedAsmSet::ContractAbi {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            // Libraries do not directly generate any asm.
            JumpOptimizedAsmSet::Library => write!(f, ""),
        }
    }
}

impl<'sc> fmt::Display for RegisterAllocatedAsmSet<'sc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterAllocatedAsmSet::ScriptMain {
                program_section,
                data_section,
            } => {
                write!(f, "{}\n{}", program_section, data_section)
            }
            RegisterAllocatedAsmSet::PredicateMain {
                program_section,
                data_section,
            } => {
                write!(f, "{}\n{}", program_section, data_section)
            }
            RegisterAllocatedAsmSet::ContractAbi {
                program_section,
                data_section,
            } => {
                write!(f, "{}\n{}", program_section, data_section)
            }
            // Libraries do not directly generate any asm.
            RegisterAllocatedAsmSet::Library => write!(f, ""),
        }
    }
}

impl<'sc> fmt::Display for FinalizedAsm<'sc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalizedAsm::ScriptMain {
                program_section,
                data_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            FinalizedAsm::PredicateMain {
                program_section,
                data_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            FinalizedAsm::ContractAbi {
                program_section,
                data_section,
            } => write!(f, "{}\n{}", program_section, data_section),
            // Libraries do not directly generate any asm.
            FinalizedAsm::Library => write!(f, ""),
        }
    }
}

impl fmt::Display for AbstractInstructionSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl<'sc> fmt::Display for InstructionSet<'sc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Default, Clone)]
pub(crate) struct AsmNamespace<'sc> {
    data_section: DataSection<'sc>,
    variables: HashMap<Ident<'sc>, VirtualRegister>,
    type_engine: Engine<'sc>,
}

/// An address which refers to a value in the data section of the asm.
#[derive(Clone, Debug)]
pub(crate) struct DataId(pub(crate) u32);

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}", self.0)
    }
}

impl<'sc> AsmNamespace<'sc> {
    pub(crate) fn resolve_type(
        &self,
        type_id: TypeId,
        error_span: &crate::Span<'sc>,
    ) -> Result<ResolvedType<'sc>, TypeError> {
        self.type_engine.resolve(type_id, error_span)
    }
    pub(crate) fn insert_variable(
        &mut self,
        var_name: Ident<'sc>,
        register_location: VirtualRegister,
    ) {
        self.variables.insert(var_name, register_location);
    }
    pub(crate) fn insert_data_value(&mut self, data: &Data<'sc>) -> DataId {
        self.data_section.insert_data_value(data)
    }
    /// Finds the register which contains variable `var_name`
    /// The `get` is unwrapped, because invalid variable expressions are
    /// checked for in the type checking stage.
    pub(crate) fn look_up_variable(
        &self,
        var_name: &Ident<'sc>,
    ) -> CompileResult<'sc, &VirtualRegister> {
        match self.variables.get(&var_name) {
            Some(o) => ok(o, vec![], vec![]),
            None => err(
                vec![],
                vec![CompileError::Internal(
                    "Unknown variable in assembly generation. This should have been an error \
                     during type checking.",
                    var_name.span.clone(),
                )],
            ),
        }
    }
}

pub(crate) fn compile_ast_to_asm<'sc>(
    ast: TypedParseTree<'sc>,
    build_config: &BuildConfig,
) -> CompileResult<'sc, FinalizedAsm<'sc>> {
    let mut register_sequencer = RegisterSequencer::new();
    let mut warnings = vec![];
    let mut errors = vec![];
    let (asm, asm_namespace) = match ast {
        TypedParseTree::Script {
            main_function,
            declarations,
            ..
        } => {
            let mut namespace: AsmNamespace = Default::default();
            let mut asm_buf = build_preamble(&mut register_sequencer).to_vec();
            check!(
                add_global_constant_decls(
                    &mut namespace,
                    &mut register_sequencer,
                    &mut asm_buf,
                    &declarations
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // start generating from the main function
            let return_register = register_sequencer.next();
            let mut body = check!(
                convert_code_block_to_asm(
                    &main_function.body,
                    &mut namespace,
                    &mut register_sequencer,
                    // TODO validate that this isn't just implicit returns?
                    Some(&return_register),
                ),
                vec![],
                warnings,
                errors
            );
            asm_buf.append(&mut body);
            asm_buf.append(&mut check!(
                ret_or_retd_value(
                    &main_function,
                    return_register,
                    &mut register_sequencer,
                    &mut namespace
                ),
                return err(warnings, errors),
                warnings,
                errors
            ));

            (
                HllAsmSet::ScriptMain {
                    program_section: AbstractInstructionSet { ops: asm_buf },
                    data_section: namespace.data_section.clone(),
                },
                namespace,
            )
        }
        TypedParseTree::Predicate {
            main_function,
            declarations,
            ..
        } => {
            let mut namespace: AsmNamespace = Default::default();
            let mut asm_buf = build_preamble(&mut register_sequencer).to_vec();
            check!(
                add_global_constant_decls(
                    &mut namespace,
                    &mut register_sequencer,
                    &mut asm_buf,
                    &declarations
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // start generating from the main function
            let mut body = check!(
                convert_code_block_to_asm(
                    &main_function.body,
                    &mut namespace,
                    &mut register_sequencer,
                    None,
                ),
                vec![],
                warnings,
                errors
            );
            asm_buf.append(&mut body);

            (
                HllAsmSet::PredicateMain {
                    program_section: AbstractInstructionSet { ops: asm_buf },
                    data_section: namespace.data_section,
                },
                namespace.clone(),
            )
        }
        TypedParseTree::Contract {
            abi_entries,
            declarations,
            ..
        } => {
            let mut namespace: AsmNamespace = Default::default();
            let mut asm_buf = build_preamble(&mut register_sequencer).to_vec();
            check!(
                add_global_constant_decls(
                    &mut namespace,
                    &mut register_sequencer,
                    &mut asm_buf,
                    &declarations
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            let (selectors_and_labels, mut contract_asm) = check!(
                compile_contract_to_selectors(abi_entries, &mut namespace, &mut register_sequencer),
                return err(warnings, errors),
                warnings,
                errors
            );
            asm_buf.append(&mut build_contract_abi_switch(
                &mut register_sequencer,
                &mut namespace,
                selectors_and_labels,
            ));
            asm_buf.append(&mut contract_asm);

            (
                HllAsmSet::ContractAbi {
                    program_section: AbstractInstructionSet { ops: asm_buf },
                    data_section: namespace.data_section.clone(),
                },
                namespace,
            )
        }
        TypedParseTree::Library { .. } => (HllAsmSet::Library, Default::default()),
    };

    if build_config.print_intermediate_asm {
        println!("{}", asm);
    }

    let finalized_asm = asm
        .remove_unnecessary_jumps()
        .allocate_registers(&asm_namespace)
        .optimize();

    if build_config.print_finalized_asm {
        println!("{}", finalized_asm);
    }

    ok(finalized_asm, warnings, errors)
}

impl<'sc> HllAsmSet<'sc> {
    pub(crate) fn remove_unnecessary_jumps(self) -> JumpOptimizedAsmSet<'sc> {
        match self {
            HllAsmSet::ScriptMain {
                data_section,
                program_section,
            } => JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section: program_section.remove_sequential_jumps(),
            },
            HllAsmSet::PredicateMain {
                data_section,
                program_section,
            } => JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section: program_section.remove_sequential_jumps(),
            },
            HllAsmSet::Library {} => JumpOptimizedAsmSet::Library,
            HllAsmSet::ContractAbi {
                data_section,
                program_section,
            } => JumpOptimizedAsmSet::ContractAbi {
                data_section,
                program_section: program_section.remove_sequential_jumps(),
            },
        }
    }
}

impl<'sc> JumpOptimizedAsmSet<'sc> {
    fn allocate_registers(self, namespace: &AsmNamespace) -> RegisterAllocatedAsmSet<'sc> {
        match self {
            JumpOptimizedAsmSet::Library => RegisterAllocatedAsmSet::Library,
            JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section,
            } => {
                let program_section = program_section
                    .realize_labels(&data_section, namespace)
                    .allocate_registers();
                RegisterAllocatedAsmSet::ScriptMain {
                    data_section,
                    program_section,
                }
            }
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => {
                let program_section = program_section
                    .realize_labels(&data_section, namespace)
                    .allocate_registers();
                RegisterAllocatedAsmSet::PredicateMain {
                    data_section,
                    program_section,
                }
            }
            JumpOptimizedAsmSet::ContractAbi {
                program_section,
                data_section,
            } => RegisterAllocatedAsmSet::ContractAbi {
                program_section: program_section
                    .realize_labels(&data_section, namespace)
                    .allocate_registers(),
                data_section,
            },
        }
    }
}

/// Represents an ASM set which has had jump labels and jumps optimized
pub enum JumpOptimizedAsmSet<'sc> {
    ContractAbi {
        data_section: DataSection<'sc>,
        program_section: AbstractInstructionSet<'sc>,
    },
    ScriptMain {
        data_section: DataSection<'sc>,
        program_section: AbstractInstructionSet<'sc>,
    },
    PredicateMain {
        data_section: DataSection<'sc>,
        program_section: AbstractInstructionSet<'sc>,
    },
    // Libraries do not generate any asm.
    Library,
}
/// Represents an ASM set which has had registers allocated
pub enum RegisterAllocatedAsmSet<'sc> {
    ContractAbi {
        data_section: DataSection<'sc>,
        program_section: InstructionSet<'sc>,
    },
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

impl<'sc> RegisterAllocatedAsmSet<'sc> {
    fn optimize(self) -> FinalizedAsm<'sc> {
        // TODO implement this -- noop for now
        match self {
            RegisterAllocatedAsmSet::Library => FinalizedAsm::Library,
            RegisterAllocatedAsmSet::ScriptMain {
                mut program_section,
                data_section,
            } => {
                // ensure there's an even number of ops so the
                // data section offset is valid
                if program_section.ops.len() & 1 != 0 {
                    program_section.ops.push(AllocatedOp {
                        opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                        comment: "word-alignment of data section".into(),
                        owning_span: None,
                    });
                }
                FinalizedAsm::ScriptMain {
                    program_section,
                    data_section,
                }
            }
            RegisterAllocatedAsmSet::PredicateMain {
                mut program_section,
                data_section,
            } => {
                // ensure there's an even number of ops so the
                // data section offset is valid
                if program_section.ops.len() & 1 != 0 {
                    program_section.ops.push(AllocatedOp {
                        opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                        comment: "word-alignment of data section".into(),
                        owning_span: None,
                    });
                }
                FinalizedAsm::PredicateMain {
                    program_section,
                    data_section,
                }
            }
            RegisterAllocatedAsmSet::ContractAbi {
                mut program_section,
                data_section,
            } => {
                // ensure there's an even number of ops so the
                // data section offset is valid
                if program_section.ops.len() & 1 != 0 {
                    program_section.ops.push(AllocatedOp {
                        opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                        comment: "word-alignment of data section".into(),
                        owning_span: None,
                    });
                }
                FinalizedAsm::ContractAbi {
                    program_section,
                    data_section,
                }
            }
        }
    }
}

pub(crate) enum NodeAsmResult<'sc> {
    JustAsm(Vec<Op<'sc>>),
    ReturnStatement { asm: Vec<Op<'sc>> },
}

/// The tuple being returned here contains the opcodes of the code block and,
/// optionally, a return register in case this node was a return statement
fn convert_node_to_asm<'sc>(
    node: &TypedAstNode<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
    // Where to put the return value of this node, if it is needed.
    return_register: Option<&VirtualRegister>,
) -> CompileResult<'sc, NodeAsmResult<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    match &node.content {
        TypedAstNodeContent::WhileLoop(r#loop) => {
            let res = check!(
                convert_while_loop_to_asm(r#loop, namespace, register_sequencer),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(NodeAsmResult::JustAsm(res), warnings, errors)
        }
        TypedAstNodeContent::Declaration(typed_decl) => {
            let res = check!(
                convert_decl_to_asm(typed_decl, namespace, register_sequencer),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(NodeAsmResult::JustAsm(res), warnings, errors)
        }
        TypedAstNodeContent::ImplicitReturnExpression(exp) => {
            // if a return register was specified, we use it. If not, we generate a register but
            // it is going to get thrown away later (in coalescing) as it is never read
            let return_register = if let Some(return_register) = return_register {
                return_register.clone()
            } else {
                register_sequencer.next()
            };
            let ops = check!(
                convert_expression_to_asm(exp, namespace, &return_register, register_sequencer),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(
                NodeAsmResult::ReturnStatement { asm: ops },
                warnings,
                errors,
            )
        }
        TypedAstNodeContent::ReturnStatement(exp) => {
            // if a return register was specified, we use it. If not, we generate a register but
            // it is going to get thrown away later (in coalescing) as it is never read
            let return_register = if let Some(return_register) = return_register {
                return_register.clone()
            } else {
                register_sequencer.next()
            };
            let ops = check!(
                convert_expression_to_asm(
                    &exp.expr,
                    namespace,
                    &return_register,
                    register_sequencer
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(
                NodeAsmResult::ReturnStatement { asm: ops },
                warnings,
                errors,
            )
        }
        TypedAstNodeContent::Expression(ref typed_expr) => {
            let return_register = if let Some(return_register) = return_register {
                return_register.clone()
            } else {
                register_sequencer.next()
            };
            let asm = check!(
                convert_expression_to_asm(
                    typed_expr,
                    namespace,
                    &return_register,
                    register_sequencer
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(NodeAsmResult::JustAsm(asm), warnings, errors)
        }
        a => {
            println!("Unimplemented: {:?}", a);
            errors.push(CompileError::Unimplemented(
                "The ASM for this construct has not been written yet.",
                node.clone().span,
            ));
            return err(warnings, errors);
        }
    }
}

/// Builds the asm preamble, which includes metadata and a jump past the metadata.
/// Right now, it looks like this:
///
/// WORD OP
/// 1    JI program_start
/// -    NOOP
/// 2    DATA_START (0-32) (in bytes, offset from $is)
/// -    DATA_START (32-64)
/// 3    LW $ds $is               1 (where 1 is in words and $is is a byte address to base off of)
/// -    ADD $ds $ds $is
/// 4    .program_start:
fn build_preamble(register_sequencer: &mut RegisterSequencer) -> [Op<'static>; 6] {
    let label = register_sequencer.get_label();
    [
        // word 1
        Op::jump_to_label(label.clone()),
        // word 1.5
        Op {
            opcode: Either::Left(VirtualOp::NOOP),
            comment: "".into(),
            owning_span: None,
        },
        // word 2 -- full word u64 placeholder
        Op {
            opcode: Either::Right(OrganizationalOp::DataSectionOffsetPlaceholder),
            comment: "data section offset".into(),
            owning_span: None,
        },
        Op::unowned_jump_label_comment(label, "end of metadata"),
        // word 3 -- load the data offset into $ds
        Op {
            opcode: Either::Left(VirtualOp::DataSectionRegisterLoadPlaceholder),
            comment: "".into(),
            owning_span: None,
        },
        // word 3.5 -- add $ds $ds $is
        Op {
            opcode: Either::Left(VirtualOp::ADD(
                VirtualRegister::Constant(ConstantRegister::DataSectionStart),
                VirtualRegister::Constant(ConstantRegister::DataSectionStart),
                VirtualRegister::Constant(ConstantRegister::InstructionStart),
            )),
            comment: "".into(),
            owning_span: None,
        },
    ]
}

/// Builds the contract switch statement, or function selector, which takes the selector
/// stored in the call frame (see https://github.com/FuelLabs/sway/issues/97#issuecomment-870150684
/// for an explanation of its location)
fn build_contract_abi_switch<'sc>(
    register_sequencer: &mut RegisterSequencer,
    namespace: &mut AsmNamespace<'sc>,
    selectors_and_labels: Vec<([u8; 4], Label)>,
) -> Vec<Op<'sc>> {
    let input_selector_register = register_sequencer.next();
    let mut asm_buf = vec![Op {
        opcode: Either::Right(OrganizationalOp::Comment),
        comment: "Begin contract ABI selector switch".into(),
        owning_span: None,
    }];
    // load the selector from the call frame
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::LW(
            input_selector_register.clone(),
            VirtualRegister::Constant(ConstantRegister::FramePointer),
            // see https://github.com/FuelLabs/fuel-specs/pull/193#issuecomment-876496372
            // We expect the last four bytes of this word to contain the selector, and the first
            // four bytes to all be 0.
            VirtualImmediate12::new_unchecked(73, "constant infallible value"),
        )),
        comment: "load input function selector".into(),
        owning_span: None,
    });

    for (selector, label) in selectors_and_labels {
        // put the selector in the data section
        let data_label = namespace.insert_data_value(&Literal::U32(u32::from_be_bytes(selector)));
        // load the data into a register for comparison
        let prog_selector_register = register_sequencer.next();
        asm_buf.push(Op {
            opcode: Either::Left(VirtualOp::LWDataId(
                prog_selector_register.clone(),
                data_label,
            )),
            comment: "load fn selector for comparison".into(),
            owning_span: None,
        });
        // compare with the input selector
        let comparison_result_register = register_sequencer.next();
        asm_buf.push(Op {
            opcode: Either::Left(VirtualOp::EQ(
                comparison_result_register.clone(),
                input_selector_register.clone(),
                prog_selector_register,
            )),
            comment: "function selector comparison".into(),
            owning_span: None,
        });

        // jump to the function label if the selector was equal
        asm_buf.push(Op {
            // if the comparison result is _not_ equal to 0, then it was indeed equal.
            opcode: Either::Right(OrganizationalOp::JumpIfNotEq(
                VirtualRegister::Constant(ConstantRegister::Zero),
                comparison_result_register,
                label,
            )),
            comment: "jump to selected function".into(),
            owning_span: None,
        });
    }

    // if none of the selectors matched, then ret
    asm_buf.push(Op {
        // see https://github.com/FuelLabs/sway/issues/97#issuecomment-875674105
        opcode: Either::Left(VirtualOp::RET(VirtualRegister::Constant(
            ConstantRegister::Zero,
        ))),
        comment: "return if no selectors matched".into(),
        owning_span: None,
    });

    asm_buf
}

fn add_global_constant_decls<'sc>(
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
    asm_buf: &mut Vec<Op<'sc>>,
    declarations: &[TypedDeclaration<'sc>],
) -> CompileResult<'sc, ()> {
    let mut warnings = vec![];
    let mut errors = vec![];
    for declaration in declarations {
        if let TypedDeclaration::ConstantDeclaration(decl) = declaration {
            let mut ops = check!(
                convert_constant_decl_to_asm(&decl, namespace, register_sequencer),
                return err(warnings, errors),
                warnings,
                errors
            );
            asm_buf.append(&mut ops);
        }
    }
    ok((), warnings, errors)
}

/// Given a contract's abi entries, compile them to jump destinations and an opcode buffer.
fn compile_contract_to_selectors<'sc>(
    abi_entries: Vec<TypedFunctionDeclaration<'sc>>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, (Vec<([u8; 4], Label)>, Vec<Op<'sc>>)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    // for every ABI function, we need:
    // 0) a jump label
    // 1) loading the argument from the call frame into the register for the function
    // 2) the function's bytecode itself
    // 3) the function selector
    let mut selectors_labels_buf = vec![];
    let mut asm_buf = vec![];
    for decl in abi_entries {
        // TODO wrapping things in a struct should be doable by the compiler eventually,
        // allowing users to pass in any number of free-floating parameters (bound by immediate limits maybe).
        // https://github.com/FuelLabs/sway/pull/115#discussion_r666466414
        if decl.parameters.len() != 4 {
            errors.push(CompileError::InvalidNumberOfAbiParams {
                span: decl.parameters_span(),
            });
            continue;
        }
        // there are currently four parameters to every ABI function, and they are required to be
        // in this order
        let cgas_name = decl.parameters[0].name.clone();
        let bal_name = decl.parameters[1].name.clone();
        let coin_color_name = decl.parameters[2].name.clone();
        let user_argument_name = decl.parameters[3].name.clone();
        // the function selector is the first four bytes of the hashed declaration/params according
        // to https://github.com/FuelLabs/sway/issues/96
        let selector = check!(decl.to_fn_selector_value(), [0u8; 4], warnings, errors);
        let fn_label = register_sequencer.get_label();
        asm_buf.push(Op::jump_label(fn_label.clone(), decl.span.clone()));
        // load the call frame argument into the function argument register
        let user_argument_register = register_sequencer.next();
        let cgas_register = register_sequencer.next();
        let bal_register = register_sequencer.next();
        let coin_color_register = register_sequencer.next();
        asm_buf.push(load_user_argument(user_argument_register.clone()));
        asm_buf.push(load_cgas(cgas_register.clone()));
        asm_buf.push(load_bal(bal_register.clone()));
        asm_buf.push(load_coin_color(coin_color_register.clone()));

        asm_buf.append(&mut check!(
            convert_abi_fn_to_asm(
                &decl,
                (user_argument_name, user_argument_register),
                (cgas_name, cgas_register),
                (bal_name, bal_register),
                (coin_color_name, coin_color_register),
                namespace,
                register_sequencer
            ),
            vec![],
            warnings,
            errors
        ));
        selectors_labels_buf.push((selector, fn_label));
    }

    ok((selectors_labels_buf, asm_buf), warnings, errors)
}
/// Given a register, load the user-provided argument into it
fn load_user_argument<'sc>(return_register: VirtualRegister) -> Op<'sc> {
    Op {
        opcode: Either::Left(VirtualOp::LW(
            return_register,
            VirtualRegister::Constant(ConstantRegister::FramePointer),
            // see https://github.com/FuelLabs/fuel-specs/pull/193#issuecomment-876496372
            VirtualImmediate12::new_unchecked(74, "infallible constant 74"),
        )),
        comment: "loading argument into abi function".into(),
        owning_span: None,
    }
}
/// Given a register, load the current value of $cgas into it
fn load_cgas<'sc>(return_register: VirtualRegister) -> Op<'sc> {
    Op {
        opcode: Either::Left(VirtualOp::LW(
            return_register,
            VirtualRegister::Constant(ConstantRegister::ContextGas),
            VirtualImmediate12::new_unchecked(0, "infallible constant 0"),
        )),
        comment: "loading cgas into abi function".into(),
        owning_span: None,
    }
}
/// Given a register, load the current value of $bal into it
fn load_bal<'sc>(return_register: VirtualRegister) -> Op<'sc> {
    Op {
        opcode: Either::Left(VirtualOp::LW(
            return_register,
            VirtualRegister::Constant(ConstantRegister::Balance),
            VirtualImmediate12::new_unchecked(0, "infallible constant 0"),
        )),
        comment: "loading coin balance into abi function".into(),
        owning_span: None,
    }
}
/// Given a register, load a pointer to the current coin color into it
fn load_coin_color<'sc>(return_register: VirtualRegister) -> Op<'sc> {
    Op {
        opcode: Either::Left(VirtualOp::LW(
            return_register,
            VirtualRegister::Constant(ConstantRegister::FramePointer),
            VirtualImmediate12::new_unchecked(5, "infallible constant 5"),
        )),
        comment: "loading coin color into abi function".into(),
        owning_span: None,
    }
}

/// Given a [TypedFunctionDeclaration] and a `return_register`, return
/// the return value of the function using either a `RET` or a `RETD` opcode.
fn ret_or_retd_value<'sc>(
    func: &TypedFunctionDeclaration<'sc>,
    return_register: VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
    namespace: &mut AsmNamespace<'sc>,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    let mut errors = vec![];
    let mut warnings = vec![];
    let mut asm_buf = vec![];
    let main_func_ret_ty: ResolvedType = todo!(
        "
     check!(
        func.return_type.force_resolution(
            &MaybeResolvedType::Resolved(ResolvedType::Unit),
            &func.return_type_span
        ),
        return err(warnings, errors),
        warnings,
        errors
    );"
    );

    if main_func_ret_ty == ResolvedType::Unit {
        // unit returns should always be zero, although because they can be
        // omitted from functions, the register is sometimes uninitialized.
        // Manually return zero in this case.
        return ok(
            vec![Op {
                opcode: Either::Left(VirtualOp::RET(VirtualRegister::Constant(
                    ConstantRegister::Zero,
                ))),
                owning_span: Some(func.return_type_span.clone()),
                comment: format!("fn {} returns unit", func.name.primary_name),
            }],
            warnings,
            errors,
        );
    }

    let size_of_main_func_return_bytes = main_func_ret_ty.stack_size_of(namespace) * 8;
    if size_of_main_func_return_bytes <= 8 {
        asm_buf.push(Op {
            owning_span: None,
            opcode: Either::Left(VirtualOp::RET(return_register)),
            comment: format!("{} fn return value", func.name.primary_name),
        });
    } else {
        // if the type is larger than one word, then we use RETD to return data
        // RB is the size_in_bytes
        let rb_register = register_sequencer.next();
        let size_bytes = namespace.insert_data_value(&Literal::U64(size_of_main_func_return_bytes));
        // `return_register` is $rA
        asm_buf.push(Op {
            opcode: Either::Left(VirtualOp::LWDataId(rb_register.clone(), size_bytes)),
            owning_span: Some(func.return_type_span.clone()),
            comment: "loading rB for RETD".into(),
        });

        // now $rB has the size of the type in bytes
        asm_buf.push(Op {
            owning_span: None,
            opcode: Either::Left(VirtualOp::RETD(return_register, rb_register)),
            comment: format!("{} fn return value", func.name.primary_name),
        });
    }
    ok(asm_buf, warnings, errors)
}
