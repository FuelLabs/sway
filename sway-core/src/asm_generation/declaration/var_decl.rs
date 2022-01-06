use crate::{
    asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer},
    asm_lang::Op,
    error::*,
    semantic_analysis::ast_node::TypedVariableDeclaration,
};

/// Provisions a register to put a variable in, and then adds the assembly used to initialize the
/// variable to the end of the buffer.
pub(crate) fn convert_variable_decl_to_asm(
    var_decl: &TypedVariableDeclaration,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let var_register = register_sequencer.next();
    let initialization =
        convert_expression_to_asm(&var_decl.body, namespace, &var_register, register_sequencer);
    namespace.insert_variable(var_decl.name.clone(), var_register);
    initialization
}
