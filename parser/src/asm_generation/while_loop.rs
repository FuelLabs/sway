use super::AsmNamespace;
use crate::semantics::ast_node::TypedWhileLoop;
use crate::vendored_vm::Opcode;
pub(super) fn convert_while_loop_to_asm(
    r#loop: TypedWhileLoop,
    namespace: &mut AsmNamespace,
) -> Vec<Opcode> {
    // convert the condition of the while loop to assembly, and then insert jump
    // instructions based on what the outcome of that condition was
    todo!()
}
