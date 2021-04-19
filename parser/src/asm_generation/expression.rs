use super::*;
use crate::semantics::TypedExpression;
use crate::vendored_vm::*;

/// Given a [TypedExpression], convert it to assembly and put its return value, if any, in the
/// `return_register`.
pub(crate) fn convert_expression_to_asm(
    exp: TypedExpression,
    namespace: &mut AsmNamespace,
    return_register: AsmRegister,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Opcode> {
    todo!()
}
