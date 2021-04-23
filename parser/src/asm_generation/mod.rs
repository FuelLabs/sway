use std::collections::HashMap;

use crate::{
    parse_tree::{AsmRegister, Literal},
    semantics::{TreeType, TypedAstNode, TypedAstNodeContent, TypedExpression, TypedParseTree},
    vendored_vm::{ImmediateValue, Op},
    Ident,
};

mod compiler_constants;
mod declaration;
mod expression;
mod register_sequencer;
mod while_loop;

pub(crate) use declaration::*;
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
pub enum HllAsmSet<'sc> {
    ContractAbi,
    ScriptMain(AbstractInstructionSet<'sc>),
    PredicateMain(AbstractInstructionSet<'sc>),
}

/// The [AbstractInstructionSet] is the list of register namespaces and operations existing
/// within those namespaces in order.
pub struct AbstractInstructionSet<'sc> {
    ops: Vec<Op<'sc>>,
}

type Data<'sc> = Literal<'sc>;
impl<'sc> AbstractInstructionSet<'sc> {}

#[derive(Default)]
pub(crate) struct DataSection<'sc> {
    /// the data to be put in the data section of the asm
    value_pairs: Vec<Data<'sc>>,
}

#[derive(Default)]
pub(crate) struct AsmNamespace<'sc> {
    data_section: DataSection<'sc>,
    variables: HashMap<Ident<'sc>, AsmRegister>,
}

impl<'sc> AsmNamespace<'sc> {
    pub(crate) fn insert_variable(&mut self, var_name: Ident<'sc>, register_location: AsmRegister) {
        self.variables.insert(var_name, register_location);
    }
    pub(crate) fn insert_data_value(&mut self, data: &Data<'sc>) -> u32 {
        self.data_section.value_pairs.push(data.clone());
        // the index of the data section where the value is stored
        (self.data_section.value_pairs.len() - 1) as u32
    }
    /// Finds the register which contains variable `var_name`
    /// The `get` is unwrapped, because invalid variable expressions are
    /// checked for in the type checking stage.
    pub(crate) fn look_up_variable(&self, var_name: &Ident<'sc>) -> &AsmRegister {
        self.variables.get(&var_name).unwrap()
    }
}

impl<'sc> HllAsmSet<'sc> {
    pub(crate) fn from_ast(ast: TypedParseTree<'sc>) -> Self {
        let mut register_sequencer = RegisterSequencer::new();
        match ast {
            TypedParseTree::Script { main_function, .. } => {
                let mut namespace: AsmNamespace = Default::default();
                let mut asm_buf = vec![];
                // start generating from the main function
                asm_buf.append(&mut convert_code_block_to_asm(
                    &main_function.body,
                    &mut namespace,
                    &mut register_sequencer,
                ));

                HllAsmSet::ScriptMain(AbstractInstructionSet { ops: asm_buf })
            }
            TypedParseTree::Predicate { main_function, .. } => {
                /*
                let mut asm_buf: Vec<Op<'sc>> = vec![];
                let mut namespace: AsmNamespace = Default::default();
                let mut asm_buf = vec![];
                // start generating from the main function
                asm_buf.append(&mut convert_fn_decl_to_asm(
                    &main_function,
                    &mut namespace,
                    &mut register_sequencer,
                ));
                */
                todo!()

                // HllAsmSet::PredicateMain(AbstractInstructionSet { ops: asm_buf })
            }
            _ => todo!(),
        }
    }
}

fn convert_node_to_asm<'sc>(
    node: &TypedAstNode<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    match &node.content {
        TypedAstNodeContent::WhileLoop(r#loop) => {
            convert_while_loop_to_asm(r#loop, namespace, register_sequencer)
        }
        TypedAstNodeContent::Declaration(typed_decl) => {
            convert_decl_to_asm(typed_decl, namespace, register_sequencer)
        }
        a => todo!("{:?}", a),
    }
}
