use std::collections::HashMap;

use crate::{
    asm_lang::{ImmediateValue, Op},
    error::*,
    parse_tree::{AsmRegister, Literal},
    semantics::{TreeType, TypedAstNode, TypedAstNodeContent, TypedExpression, TypedParseTree},
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

/// An address which refers to a value in the data section of the asm.
pub(crate) struct DataId(u32);

impl<'sc> AsmNamespace<'sc> {
    pub(crate) fn insert_variable(&mut self, var_name: Ident<'sc>, register_location: AsmRegister) {
        self.variables.insert(var_name, register_location);
    }
    pub(crate) fn insert_data_value(&mut self, data: &Data<'sc>) -> DataId {
        self.data_section.value_pairs.push(data.clone());
        // the index of the data section where the value is stored
        DataId((self.data_section.value_pairs.len() - 1) as u32)
    }
    /// Finds the register which contains variable `var_name`
    /// The `get` is unwrapped, because invalid variable expressions are
    /// checked for in the type checking stage.
    pub(crate) fn look_up_variable(
        &self,
        var_name: &Ident<'sc>,
    ) -> CompileResult<'sc, &AsmRegister> {
        match self.variables.get(&var_name) {
            Some(o) => ok(o, vec![], vec![]),
            None => err(vec![], vec![CompileError::Internal ("Unknown variable in assembly generation. This should have been an error during type checking.",  var_name.span.clone() )])

        }
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
                    None,
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
    return_register: Option<&AsmRegister>,
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
        a => todo!("{:?}", a),
    }
}
