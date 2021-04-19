use crate::{
    parse_tree::AsmRegister,
    semantics::{TreeType, TypedAstNode, TypedAstNodeContent, TypedExpression, TypedParseTree},
    vendored_vm::Op,
};

mod compiler_constants;
mod expression;
mod register_sequencer;
mod while_loop;

pub(crate) use expression::*;
pub(crate) use register_sequencer::*;
pub(crate) use while_loop::*;

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
pub(crate) enum HllAsmSet<'sc> {
    ContractAbi,
    ScriptMain(AbstractInstructionSet<'sc>),
    PredicateMain(AbstractInstructionSet<'sc>),
}

/// The [AbstractInstructionSet] is the list of register namespaces and operations existing
/// within those namespaces in order.
pub(crate) struct AbstractInstructionSet<'sc> {
    /// Used to store mappings of values to register locations
    namespace: AsmNamespace,
    asm: Vec<Op<'sc>>,
}

#[derive(Default)]
pub(crate) struct AsmNamespace {}

impl<'sc> AbstractInstructionSet<'sc> {
    pub(crate) fn from_ast(ast: TypedParseTree<'sc>, tree_type: TreeType) -> Self {
        let mut register_sequencer = RegisterSequencer::new();
        match tree_type {
            TreeType::Script | TreeType::Predicate => {
                let mut namespace: AsmNamespace = Default::default();
                for node in ast.root_nodes {
                    let asm = convert_node_to_asm(node, &mut namespace, &mut register_sequencer);
                }
            }
            TreeType::Contract => todo!(),
            TreeType::Library => todo!(),
        }
        todo!()
    }
}

fn convert_node_to_asm<'sc>(
    node: TypedAstNode<'sc>,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    match node.content {
        TypedAstNodeContent::WhileLoop(r#loop) => {
            convert_while_loop_to_asm(r#loop, namespace, register_sequencer)
        }
        a => todo!("{:?}", a),
    }
}

/*
fn allocate_registers(bytecode: VirtualizedByteCode) -> FinalizedByteCode {
    const MAX_REGISTERS = 48;
    let mut allocated_registers = Vec::new();
    for AsmOp { registers, .. } in bytecode {
        for register in registers {
            if !allocated_registers.contains(register) {
                if allocated_registers.len() == MAX_REGISTERS {
                    panic!("Out of registers!");
                } else {
                    allocated_registers.push(register.clone());
                    let register_name = format!("r{}", allocated_registers.len());
                    // TODOthis should be some sort of mapping
                }
            }
        }
    }

}
*/
