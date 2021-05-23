use std::{collections::HashMap, fmt};

use crate::{
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
    semantic_analysis::{TypedAstNode, TypedAstNodeContent, TypedParseTree},
    Ident,
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
    ContractAbi,
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
    fn realize_labels(self, data_section: &DataSection) -> RealizedAbstractInstructionSet<'sc> {
        let mut label_namespace: HashMap<&Label, u64> = Default::default();
        let mut counter = 0;
        for op in &self.ops {
            match op.opcode {
                Either::Right(OrganizationalOp::Label(ref lab)) => {
                    label_namespace.insert(lab, counter);
                }
                // these ops will end up being exactly one op, so the counter goes up one
                Either::Right(OrganizationalOp::Ld(..)) => counter += 2,
                Either::Right(OrganizationalOp::Jump(..))
                | Either::Right(OrganizationalOp::JumpIfNotEq(..))
                | Either::Left(_) => {
                    counter += 1;
                }
                Either::Right(OrganizationalOp::Comment) => (),
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
                    OrganizationalOp::Ld(reg, data_lab) => {
                        let data = data_section.value_pairs[data_lab.0 as usize].clone();
                        // TODO force_to_imm() is very very bad. see it for details
                        realized_ops.push(RealizedOp {
                            opcode: VirtualOp::ORI(
                                reg,
                                VirtualRegister::Constant(ConstantRegister::Zero),
                                data.force_to_imm(),
                            ),
                            owning_span,
                            comment,
                        });
                    }
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
                    OrganizationalOp::Comment => continue,
                    OrganizationalOp::Label(..) => continue,
                },
            };
        }
        RealizedAbstractInstructionSet { ops: realized_ops }
    }
}

struct RegisterAllocationStatus {
    reg: AllocatedRegister,
    in_use: Option<VirtualRegister>,
}
pub(crate) struct RegisterPool {
    registers: Vec<RegisterAllocationStatus>,
}

impl RegisterPool {
    fn init() -> Self {
        let register_pool: Vec<RegisterAllocationStatus> = (0
            ..compiler_constants::NUM_FREE_REGISTERS)
            .rev()
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
        // find the next unused register, return it, flip assign it
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
    value_pairs: Vec<Data<'sc>>,
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
                Literal::Byte32(b) => format!(
                    ".byte32 0x{}",
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
            HllAsmSet::ContractAbi { .. } => write!(f, "TODO contract ABI asm is unimplemented"),
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
            JumpOptimizedAsmSet::ContractAbi { .. } => {
                write!(f, "TODO contract ABI asm is unimplemented")
            }
            // Libraries do not directly generate any asm.
            JumpOptimizedAsmSet::Library => write!(f, ""),
        }
    }
}

impl<'sc> fmt::Display for RegisterAllocatedAsmSet<'sc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterAllocatedAsmSet::ScriptMain { program_section } => {
                write!(f, "{}", program_section)
            }
            RegisterAllocatedAsmSet::PredicateMain { program_section } => {
                write!(f, "{}", program_section)
            }
            RegisterAllocatedAsmSet::ContractAbi { .. } => {
                write!(f, "TODO contract ABI asm is unimplemented")
            }
            // Libraries do not directly generate any asm.
            RegisterAllocatedAsmSet::Library => write!(f, ""),
        }
    }
}

impl<'sc> fmt::Display for FinalizedAsm<'sc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalizedAsm::ScriptMain { program_section } => write!(f, "{}", program_section,),
            FinalizedAsm::PredicateMain { program_section } => write!(f, "{}", program_section,),
            FinalizedAsm::ContractAbi { .. } => {
                write!(f, "TODO contract ABI asm is unimplemented")
            }
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
}

/// An address which refers to a value in the data section of the asm.
#[derive(Clone)]
pub(crate) struct DataId(u32);

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}", self.0)
    }
}

impl<'sc> AsmNamespace<'sc> {
    pub(crate) fn insert_variable(
        &mut self,
        var_name: Ident<'sc>,
        register_location: VirtualRegister,
    ) {
        self.variables.insert(var_name, register_location);
    }
    pub(crate) fn insert_data_value(&mut self, data: &Data<'sc>) -> DataId {
        // if there is an identical data value, use the same id
        match self.data_section.value_pairs.iter().position(|x| x == data) {
            Some(num) => DataId(num as u32),
            None => {
                self.data_section.value_pairs.push(data.clone());
                // the index of the data section where the value is stored
                DataId((self.data_section.value_pairs.len() - 1) as u32)
            }
        }
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
) -> CompileResult<'sc, FinalizedAsm> {
    let mut register_sequencer = RegisterSequencer::new();
    let mut warnings = vec![];
    let mut errors = vec![];
    let asm = match ast {
        TypedParseTree::Script { main_function, .. } => {
            let mut namespace: AsmNamespace = Default::default();
            let mut asm_buf = vec![];
            // start generating from the main function
            let mut body = type_check!(
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

            HllAsmSet::ScriptMain {
                program_section: AbstractInstructionSet { ops: asm_buf },
                data_section: namespace.data_section,
            }
        }
        TypedParseTree::Predicate { main_function, .. } => {
            let mut namespace: AsmNamespace = Default::default();
            let mut asm_buf = vec![];
            // start generating from the main function
            let mut body = type_check!(
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

            HllAsmSet::PredicateMain {
                program_section: AbstractInstructionSet { ops: asm_buf },
                data_section: namespace.data_section,
            }
        }
        TypedParseTree::Contract { .. } => {
            unimplemented!("Contract ABI ASM generation has not been implemented.");
        }
        TypedParseTree::Library { .. } => HllAsmSet::Library,
    };

    ok(
        asm.remove_unnecessary_jumps()
            .allocate_registers()
            .optimize(),
        warnings,
        errors,
    )
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
            HllAsmSet::ContractAbi {} => JumpOptimizedAsmSet::ContractAbi {},
        }
    }
}

impl<'sc> JumpOptimizedAsmSet<'sc> {
    fn allocate_registers(self) -> RegisterAllocatedAsmSet<'sc> {
        // TODO implement this -- noop for now
        match self {
            JumpOptimizedAsmSet::Library => RegisterAllocatedAsmSet::Library,
            JumpOptimizedAsmSet::ScriptMain {
                data_section,
                program_section,
            } => RegisterAllocatedAsmSet::ScriptMain {
                program_section: program_section
                    .clone()
                    .realize_labels(&data_section)
                    .allocate_registers(),
            },
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => RegisterAllocatedAsmSet::PredicateMain {
                program_section: program_section
                    .realize_labels(&data_section)
                    .allocate_registers(),
            },
            JumpOptimizedAsmSet::ContractAbi => RegisterAllocatedAsmSet::ContractAbi,
        }
    }
}

/// Represents an ASM set which has had jump labels and jumps optimized
pub enum JumpOptimizedAsmSet<'sc> {
    ContractAbi,
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
    ContractAbi,
    ScriptMain {
        program_section: InstructionSet<'sc>,
    },
    PredicateMain {
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
            RegisterAllocatedAsmSet::ScriptMain { program_section } => {
                FinalizedAsm::ScriptMain { program_section }
            }
            RegisterAllocatedAsmSet::PredicateMain { program_section } => {
                FinalizedAsm::PredicateMain { program_section }
            }
            RegisterAllocatedAsmSet::ContractAbi => FinalizedAsm::ContractAbi,
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
            let res = type_check!(
                convert_while_loop_to_asm(r#loop, namespace, register_sequencer),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(NodeAsmResult::JustAsm(res), warnings, errors)
        }
        TypedAstNodeContent::Declaration(typed_decl) => {
            let res = type_check!(
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
            let ops = type_check!(
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
        _ => {
            errors.push(CompileError::Unimplemented(
                "The ASM for this construct has not been written yet.",
                node.clone().span,
            ));
            return err(warnings, errors);
        }
    }
}
