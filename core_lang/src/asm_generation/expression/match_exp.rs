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
    1. condition evaluation
    2. conditional jump to 1st branch
        conditional jump to 2nd branch
        ...
        conditional jump to nth branch
    3. 1st branch label
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
    4. after last branch label
    */

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
    let patterns_results: Vec<VirtualRegister> = branches
        .iter()
        .map(|_| register_sequencer.next())
        .collect();
    for i in 0..branches.len() {
        let branch = branches[i].clone();
        let pattern_result = patterns_results[i].clone();
        let branch = todo!();
        // if the condition is not false, jump to the approporiate branch
        asm_buf.push(Op::jump_if_not_equal(
            pattern_result.clone(),
            VirtualRegister::Constant(ConstantRegister::Zero),
            branches_labels[i]
        ));
    }
    let branches_results: Vec<VirtualRegister> = branches
        .iter()
        .map(|_| register_sequencer.next())
        .collect();
    for i in 0..branches.len() {
        let branch = branches[i].clone();
        let branch_result = branches_results[i].clone();
        let branch_label = branches_labels[i].clone();
        let mut branch2 = check!(
            convert_expression_to_asm(&branches[i].result, namespace, &branch_result, register_sequencer),
            return err(warnings, errors),
            warnings,
            errors
        );
        asm_buf.push(Op::jump_label_comment(
            branch_label,
            branch.clone().result.span,
            format!("end of branch {:?}", i),
        ));
        asm_buf.append(&mut branch2);
        asm_buf.push(Op::register_move(
            return_register.clone(),
            branch_result,
            branch.clone().result.span
        ));
        asm_buf.push(Op::jump_to_label_comment(
            after_branches_label.clone(),
            format!("end of branch {:?}", i),
        ));
    }

    asm_buf.push(Op::unowned_jump_label_comment(
        after_branches_label,
        "end of match exp",
    ));

    ok(asm_buf, warnings, errors)
}
