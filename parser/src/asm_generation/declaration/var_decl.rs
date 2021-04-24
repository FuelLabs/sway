use crate::{
    asm_generation::{
        convert_expression_to_asm, AsmNamespace, ExpressionAsmResult, RegisterSequencer,
    },
    semantics::ast_node::TypedVariableDeclaration,
    vendored_vm::Op,
};

pub(crate) fn convert_variable_decl_to_asm<'sc>(
    var_decl: &TypedVariableDeclaration<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    let var_register = register_sequencer.next();
    let initialization =
        convert_expression_to_asm(&var_decl.body, namespace, &var_register, register_sequencer);
    match initialization {
        ExpressionAsmResult::Ops(ops) => {
            namespace.insert_variable(var_decl.name.clone(), var_register);
            ops
        }
        ExpressionAsmResult::Shortcut(new_return_register) => {
            // This means the variable already has been initialized to a register so we just return
            // that register
            namespace.insert_variable(var_decl.name.clone(), new_return_register);
            vec![]
        }
    }
}
