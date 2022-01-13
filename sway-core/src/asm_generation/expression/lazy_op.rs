use crate::{
    asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer},
    asm_lang::{ConstantRegister, Op, VirtualRegister},
    error::*,
    parse_tree::LazyOp,
    semantic_analysis::TypedExpression,
    CompileResult,
};

pub(crate) fn convert_lazy_operator_to_asm(
    op: &LazyOp,
    lhs: &TypedExpression,
    rhs: &TypedExpression,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    // Short circuiting operators need to evaluate the LHS first, and then only conditionally
    // evaluate the RHS.
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_ops = vec![];

    // Always evaluate the LHS.  Put the result into the return register since with short
    // circuiting it might be all we need to do.
    let mut lhs_asm_ops = check!(
        convert_expression_to_asm(lhs, namespace, return_register, register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    );
    asm_ops.append(&mut lhs_asm_ops);

    // Depending on the operator we can skip evaluating the RHS using JNEI.  For && if the LHS is
    // false we skip: if it's != true.  For || if the LHS is true we skip: if it's != false.
    let comparison_reg = match op {
        LazyOp::And => ConstantRegister::One,
        LazyOp::Or => ConstantRegister::Zero,
    };
    let skip_label = register_sequencer.get_label();
    let mut jnei_op = Op::jump_if_not_equal(
        return_register.clone(),
        VirtualRegister::Constant(comparison_reg),
        skip_label.clone(),
    );
    jnei_op.comment = "conditionally skip RHS for lazy operator".to_owned();
    asm_ops.push(jnei_op);

    // Evaluate the RHS.  Again, we can put the result into the return register as it will be the
    // final value we want.
    let mut rhs_asm_ops = check!(
        convert_expression_to_asm(rhs, namespace, return_register, register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    );
    asm_ops.append(&mut rhs_asm_ops);

    // Finally add the skip label.
    asm_ops.push(Op::unowned_jump_label(skip_label));

    ok(asm_ops, warnings, errors)
}
