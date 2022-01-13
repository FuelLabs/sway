use super::*;
use crate::asm_lang::{ConstantRegister, VirtualRegister};
use crate::semantic_analysis::ast_node::TypedWhileLoop;
pub(super) fn convert_while_loop_to_asm(
    r#loop: &TypedWhileLoop,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut buf: Vec<Op> = vec![];
    // A while loop consists of (in order of asm):
    // 0. A label to jump to
    // 1. Evaluate the condition
    // 2. Branch based on condition
    // 3. Loop Body
    // 4. Jump to beginning label
    // 5. Exit label

    // step 0
    let label = register_sequencer.get_label();
    let exit_label = register_sequencer.get_label();
    let condition_span = r#loop.condition.span.clone();
    buf.push(Op::jump_label_comment(
        label.clone(),
        condition_span,
        "begin while loop",
    ));

    // step 1
    let condition_result_register = register_sequencer.next();
    let mut asm_for_condition = check!(
        convert_expression_to_asm(
            &r#loop.condition,
            namespace,
            &condition_result_register,
            register_sequencer,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    buf.append(&mut asm_for_condition);

    // step 2
    // compare the result to FALSE
    // if it is FALSE, then jump to the end of the block.
    buf.push(Op::jump_if_not_equal(
        condition_result_register,
        VirtualRegister::Constant(ConstantRegister::One),
        exit_label.clone(),
    ));

    // the implicit return value of a while loop block, if any, should be ignored,
    // so we pass None into the final argument of code block conversion
    // step 3: run the loop body
    let mut body = check!(
        convert_code_block_to_asm(&r#loop.body, namespace, register_sequencer, None),
        vec![],
        warnings,
        errors
    );
    buf.append(&mut body);

    // step 4: jump back to beginning to re-evaluate the condition
    buf.push(Op::jump_to_label(label));

    // step 5
    buf.push(Op::jump_label_comment(
        exit_label,
        r#loop.body.whole_block_span.clone(),
        "exit while loop",
    ));

    ok(buf, warnings, errors)
}
