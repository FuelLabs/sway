use std::{collections::HashMap, fmt};

use crate::{
    asm_lang::{Label, Op, OrganizationalOp, RegisterId},
    error::*,
    parse_tree::Literal,
    semantic_analysis::{TypedAstNode, TypedAstNodeContent, TypedParseTree},
    Ident,
};
use either::Either;

mod compiler_constants;
mod declaration;
mod expression;
mod register_sequencer;
mod while_loop;

pub(crate) use declaration::*;
pub(crate) use expression::*;
pub(crate) use register_sequencer::*;

use while_loop::convert_while_loop_to_asm;

// Initially, the bytecode will have a lot of individual registers being used. Each register will
// have a new unique identifier. For example, two separate invocations of `+` will result in 4
// registers being used for arguments and 2 for outputs.
//
// After that, the level 0 bytecode will go through a process where register use is minified, producing level 1 bytecode. This process
// is as such:
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
/// An [InstructionSet] is produced by allocating registers on an [AbstractInstructionSet].
pub struct InstructionSet<'sc> {
    ops: Vec<Op<'sc>>,
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

    fn allocate_registers(mut self) -> InstructionSet<'sc> {
        // Eventually, we will use a cool graph-coloring algorithm.
        // For now, just keep a pool of registers and return
        // registers when they are not read anymore

        // construct a mapping from every op to the registers it uses
        let mut op_register_mapping = self
            .ops
            .iter_mut()
            .map(|op| {
                (
                    op.clone(),
                    match op.opcode {
                        Either::Left(mut opc) => opc.registers(),
                        Either::Right(mut orgop) => orgop.registers(),
                    },
                )
            })
            .collect::<Vec<_>>();

        // get registers from the pool.
        // if the registers are never read again, return them to the pool.
        let mut pool = RegisterPool::init();
        for (op, registers) in op_register_mapping {
            let new_registers: Option<Vec<_>> = registers
                .into_iter()
                .map(|reg| (reg, pool.get_register()))
                .collect();
            let new_registers = match new_registers {
                a @ (_, Some(_)) => a,
                _ => todo!("Return out of registers error"),
            };
            // if the virtual register is never read again, then we can
            // return this virtual register back into the pool

            // TODO:
            // properly parse reserved registers and handle them in asm expressions
            // do not pull from the pool for reserved registers
            //
        }
        todo!()
    }
}

fn register_is_never_read_again(reg: &Register, ops: &[(Op, Vec<Register>)]) -> bool {
    todo!()
}
struct RegisterPool {
    available_registers: Vec<Register>,
}

enum Register {
    Free(u8),
    Reserved(u8),
}

impl RegisterPool {
    fn init() -> Self {
        let mut register_pool: Vec<Register> = (compiler_constants::NUM_FREE_REGISTERS..0)
            .map(|x| Register::Free(x))
            .collect();
        Self {
            available_registers: register_pool,
        }
    }

    fn get_register(&mut self) -> Option<Register> {
        self.available_registers.pop()
    }

    fn return_register_to_pool(&mut self, item_to_return: Register) {
        self.available_registers.push(item_to_return);
    }
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
            } => write!(f, "{}\n{}", data_section, program_section),
            HllAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", data_section, program_section),
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
            } => write!(f, "{}\n{}", data_section, program_section),
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", data_section, program_section),
            JumpOptimizedAsmSet::ContractAbi { .. } => {
                write!(f, "TODO contract ABI asm is unimplemented")
            }
            // Libraries do not directly generate any asm.
            JumpOptimizedAsmSet::Library => write!(f, ""),
        }
    }
}

impl fmt::Display for RegisterAllocatedAsmSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterAllocatedAsmSet::ScriptMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", data_section, program_section),
            RegisterAllocatedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", data_section, program_section),
            RegisterAllocatedAsmSet::ContractAbi { .. } => {
                write!(f, "TODO contract ABI asm is unimplemented")
            }
            // Libraries do not directly generate any asm.
            RegisterAllocatedAsmSet::Library => write!(f, ""),
        }
    }
}

impl fmt::Display for FinalizedAsm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalizedAsm::ScriptMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", data_section, program_section),
            FinalizedAsm::PredicateMain {
                data_section,
                program_section,
            } => write!(f, "{}\n{}", data_section, program_section),
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

impl fmt::Display for InstructionSet<'_> {
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
    variables: HashMap<Ident<'sc>, RegisterId>,
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
    pub(crate) fn insert_variable(&mut self, var_name: Ident<'sc>, register_location: RegisterId) {
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
    ) -> CompileResult<'sc, &RegisterId> {
        match self.variables.get(&var_name) {
            Some(o) => ok(o, vec![], vec![]),
            None => err(vec![], vec![CompileError::Internal ("Unknown variable in assembly generation. This should have been an error during type checking.",  var_name.span.clone() )])

        }
    }
}

pub(crate) fn compile_ast_to_asm<'sc>(
    ast: TypedParseTree<'sc>,
) -> CompileResult<'sc, FinalizedAsm<'sc>> {
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
                data_section,
                program_section: program_section.clone().allocate_registers(),
            },
            JumpOptimizedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => RegisterAllocatedAsmSet::PredicateMain {
                data_section,
                program_section: program_section.allocate_registers(),
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
                data_section,
                program_section,
            } => FinalizedAsm::ScriptMain {
                data_section,
                program_section,
            },
            RegisterAllocatedAsmSet::PredicateMain {
                data_section,
                program_section,
            } => FinalizedAsm::PredicateMain {
                data_section,
                program_section,
            },
            RegisterAllocatedAsmSet::ContractAbi => FinalizedAsm::ContractAbi,
        }
    }
}

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
    return_register: Option<&RegisterId>,
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
