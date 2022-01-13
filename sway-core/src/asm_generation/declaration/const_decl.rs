use crate::{
    asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer},
    asm_lang::Op,
    error::*,
    semantic_analysis::ast_node::TypedConstantDeclaration,
};

/// Provisions a register to put a value in, and then adds the assembly used to initialize the
/// value to the end of the buffer.
pub(crate) fn convert_constant_decl_to_asm(
    const_decl: &TypedConstantDeclaration,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let val_register = register_sequencer.next();
    let initialization = convert_expression_to_asm(
        &const_decl.value,
        namespace,
        &val_register,
        register_sequencer,
    );
    namespace.insert_variable(const_decl.name.clone(), val_register);
    initialization
}
