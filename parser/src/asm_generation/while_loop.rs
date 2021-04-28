use super::*;
use crate::asm_lang::Op;
use crate::semantics::ast_node::TypedWhileLoop;
pub(super) fn convert_while_loop_to_asm<'sc>(
    r#loop: &TypedWhileLoop<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    let mut buf: Vec<Op> = vec![];
    // convert the condition of the while loop to assembly, and then insert jump
    // instructions based on what the outcome of that condition was
    let condition_result_register = register_sequencer.next();
    let condition_span = r#loop.condition.span.clone();
    let asm_for_condition = convert_expression_to_asm(
        &r#loop.condition,
        namespace,
        &condition_result_register,
        register_sequencer,
    );

    let condition_ops = vec![Op::new_with_comment(
        todo!(),
        todo!(),
        "Check if while loop condition is true",
    )];

    todo!()
}
