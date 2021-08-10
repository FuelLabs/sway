use super::*;
use crate::error::*;
use crate::semantic_analysis::ast_node::*;
use either::Either;
/// Converts a function application of a contract ABI function into assembly
pub(crate) fn convert_contract_call_to_asm<'sc>(
    metadata: &ContractCallMetadata<'sc>,
    cgas: &TypedExpression<'sc>,
    bal: &TypedExpression<'sc>,
    coin_color: &TypedExpression<'sc>,
    user_argument: &TypedExpression<'sc>,
    register_sequencer: &mut RegisterSequencer,
    namespace: &mut AsmNamespace<'sc>,
    span: Span<'sc>,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    // step 0. evaluate the arguments
    // step 1. construct the CALL op using the arguments
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![];
    // step 0
    //
    let user_argument_register = register_sequencer.next();
    let gas_to_forward = register_sequencer.next();
    let bal_register = register_sequencer.next();
    let coin_color_register = register_sequencer.next();
    asm_buf.append(&mut type_check!(
        convert_expression_to_asm(
            user_argument,
            namespace,
            &user_argument_register,
            register_sequencer
        ),
        vec![],
        warnings,
        errors
    ));
    asm_buf.append(&mut type_check!(
        convert_expression_to_asm(cgas, namespace, &gas_to_forward, register_sequencer),
        vec![],
        warnings,
        errors
    ));
    asm_buf.append(&mut type_check!(
        convert_expression_to_asm(bal, namespace, &bal_register, register_sequencer),
        vec![],
        warnings,
        errors
    ));
    asm_buf.append(&mut type_check!(
        convert_expression_to_asm(
            coin_color,
            namespace,
            &coin_color_register,
            register_sequencer
        ),
        vec![],
        warnings,
        errors
    ));
    // Write to memory, in order: the contract address (32 bytes), the function selector (param1, 8
    // bytes), and the user argument (param2, 8 bytes).

    let contract_address = todo!(
        "set up: contract address, selector, then user param in memory. \
        make this a register containing a pointer to the beginning of that sequence."
    );
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::CALL(
            contract_address,
            bal_register,
            coin_color_register,
            gas_to_forward,
        )),
        comment: "call external contract".into(),
        owning_span: Some(span),
    });

    ok(asm_buf, warnings, errors)
}
