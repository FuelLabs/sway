use crate::asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer};
use crate::asm_lang::{
    virtual_ops::{ConstantRegister, VirtualRegister, Label},
    Op,
};
use crate::error::*;

use crate::semantic_analysis::{TypedExpression, ast_node::TypedMatchBranch};


use crate::CompileResult;

pub(crate) fn convert_match_exp_to_asm<'sc>(
    primary_expression: &TypedExpression<'sc>,
    branches: &Vec<TypedMatchBranch<'sc>>,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    /*
    the asm order is:
    condition evaluation
    conditional jump to 1st branch
    conditional jump to 2nd branch
    ...
    conditional jump to nth branch
    1st branch label
    1st branch
    move 1st branch result to return register
    jump to after last branch
    2nd branch label
    2nd branch
    move 2nd branch result to return register
    jump to after last branch
    ...
    n-1 branch label
    n-1 branch
    move n-1 branch result to return register
    jump to after last branch
    nth branch label
    nth branch
    move nth branch result to return register
    after last branch label
    */


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

    let branches_labels: Vec<Label> = branches
        .iter()
        .map(|_| register_sequencer.get_label())
        .collect();
    let after_branches_label = register_sequencer.get_label();
    let primary_expression_result = register_sequencer.next();
    let mut primary_expression = check!(
        convert_expression_to_asm(primary_expression, namespace, &primary_expression_result, register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    );
    asm_buf.push(Op::new_comment("begin match expression"));
    asm_buf.append(&mut primary_expression);
    for i in 0..branches.len() {
        // if the condition is not false, jump to the approporiate branch
        asm_buf.push(Op::jump_if_not_equal(
            primary_expression_result.clone(),
            VirtualRegister::Constant(ConstantRegister::Zero),
            branches_labels[i]
        ));
    }
    let branches_results: Vec<VirtualRegister> = branches
        .iter()
        .map(|_| register_sequencer.next())
        .collect();

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
            convert_expression_to_asm(&r#else, namespace, &else_branch_result, register_sequencer),
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
