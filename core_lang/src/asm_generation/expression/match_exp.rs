use crate::asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer};
use crate::asm_lang::{
    virtual_ops::{ConstantRegister, Label, VirtualRegister},
    Op,
};
use crate::error::*;

use crate::semantic_analysis::{
    ast_node::{PatternVariant, TypedMatchBranch, TypedMatchPattern, TypedExpressionVariant},
    TypedExpression,
};

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

    // construct labels to refer to each branch
    let branches_labels: Vec<Label> = branches
        .iter()
        .map(|_| register_sequencer.get_label())
        .collect();

    // construct label to refer to after the last branch
    let after_branches_label = register_sequencer.get_label();

    // construct registers that pattern results go in
    let pattern_match_results: Vec<VirtualRegister> =
        branches.iter().map(|_| register_sequencer.next()).collect();

    // transform pattern matches to asm
    let mut pattern_matches = vec![];
    for i in 0..branches.len() {
        pattern_matches.push(check!(
            convert_pattern_to_asm(
                &branches[i].clone().pattern,
                primary_expression,
                &pattern_match_results[i].clone(),
                namespace,
                register_sequencer
            ),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }

    // push pattern matches onto asm_buf
    asm_buf.push(Op::new_comment("begin match expression"));
    for i in 0..branches.len() {
        asm_buf.append(&mut pattern_matches[i].clone());
        // if the condition is not false, jump to the approporiate branch
        asm_buf.push(Op::jump_if_not_equal(
            pattern_match_results[i].clone(),
            VirtualRegister::Constant(ConstantRegister::Zero),
            branches_labels[i].clone(),
        ));
    }

    // construct registers that branch results go in
    let branches_results: Vec<VirtualRegister> =
        branches.iter().map(|_| register_sequencer.next()).collect();

    for i in 0..branches.len() {
        let span = branches[i].clone().result.span;
        // convert branches to asm
        let mut branch = check!(
            convert_expression_to_asm(
                &branches[i].result,
                namespace,
                &branches_results[i].clone(),
                register_sequencer
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        // push branch labels onto asm_buf
        asm_buf.push(Op::jump_label_comment(
            branches_labels[i].clone(),
            span.clone(),
            format!("end of branch {:?}", i),
        ));
        // push branch asm onto asm_bug
        asm_buf.append(&mut branch);
        // push onto asm_buf moving the result to the return register
        asm_buf.push(Op::register_move(
            return_register.clone(),
            branches_results[i].clone(),
            span,
        ));
        // push a jump to the after branches label to asm_buf
        asm_buf.push(Op::jump_to_label_comment(
            after_branches_label.clone(),
            format!("end of branch {:?}", i),
        ));
    }

    // push the after branches label to asm_buf
    asm_buf.push(Op::unowned_jump_label_comment(
        after_branches_label,
        "end of match exp",
    ));

    ok(asm_buf, warnings, errors)
}

pub(crate) fn convert_pattern_to_asm<'sc>(
    pattern: &TypedMatchPattern<'sc>,
    primary_expression: &TypedExpression<'sc>,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    match pattern.pattern {
        PatternVariant::CatchAll => todo!(),
        PatternVariant::Expression(pattern_exp) => {
            match (pattern_exp.expression, primary_expression.expression) {
                (
                    TypedExpressionVariant::Literal(pattern_lit),
                    TypedExpressionVariant::Literal(primary_lit),
                ) => todo!(),
                _ => unimplemented!(),
            }
        }
    }
}
