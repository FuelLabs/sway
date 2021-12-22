use crate::asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer};
use crate::asm_lang::{ConstantRegister, Op, VirtualRegister};
use crate::error::*;

use crate::semantic_analysis::TypedExpression;

use crate::CompileResult;

pub(crate) fn convert_if_exp_to_asm(
    condition: &TypedExpression,
    then: &TypedExpression,
    r#else: &Option<Box<TypedExpression>>,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    // step 0: construct 2 jump labels: to the else, and to after the else.
    // step 1: evaluate the condition
    // step 2: conditional jump -- if the condition is false, jump to the else label. If there is no
    // else, jump to the end. step 2: add jump to after the else from the end of the `then`
    // branch
    //
    // to recap, the asm order is: condition evaluation,
    //         conditional jump to else or after else,
    //         then branch,
    //         move then result to return register
    //         jump to after else,
    //         else branch label
    //         else branch,
    //         move else result to return register
    //         after else branch label

    // step 3: put return value from whatever branch was evaluated into the return register
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![];

    let else_label = register_sequencer.get_label();
    let after_else_label = register_sequencer.get_label();
    let condition_result = register_sequencer.next();
    let mut condition = check!(
        convert_expression_to_asm(condition, namespace, &condition_result, register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    );
    asm_buf.push(Op::new_comment("begin if expression"));

    asm_buf.append(&mut condition);
    // if the condition is not true, jump to the else branch (if there is one).
    asm_buf.push(Op::jump_if_not_equal(
        condition_result.clone(),
        VirtualRegister::Constant(ConstantRegister::One),
        if r#else.is_some() {
            else_label.clone()
        } else {
            after_else_label.clone()
        },
    ));

    let then_branch_result = register_sequencer.next();
    let mut then_branch = check!(
        convert_expression_to_asm(then, namespace, &then_branch_result, register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    );
    asm_buf.append(&mut then_branch);
    // move the result of the then branch into the return register
    asm_buf.push(Op::register_move(
        return_register.clone(),
        then_branch_result,
        then.clone().span,
    ));
    asm_buf.push(Op::jump_to_label_comment(
        after_else_label.clone(),
        "end of then branch",
    ));
    if let Some(r#else) = r#else {
        asm_buf.push(Op::jump_label_comment(
            else_label,
            r#else.span.clone(),
            "beginning of else branch",
        ));
        let else_branch_result = register_sequencer.next();
        let mut else_branch = check!(
            convert_expression_to_asm(r#else, namespace, &else_branch_result, register_sequencer),
            return err(warnings, errors),
            warnings,
            errors
        );
        asm_buf.append(&mut else_branch);

        // move the result of the else branch into the return register
        asm_buf.push(Op::register_move(
            return_register.clone(),
            else_branch_result,
            r#else.clone().span,
        ));
    }

    asm_buf.push(Op::unowned_jump_label_comment(
        after_else_label,
        "End of if exp",
    ));

    ok(asm_buf, warnings, errors)
}
