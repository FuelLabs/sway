use super::*;
use crate::semantic_analysis::ast_node::*;
use either::Either;
/// Converts a function application of a contract ABI function into assembly
#[allow(clippy::too_many_arguments)]
pub(crate) fn convert_contract_call_to_asm<'sc>(
    metadata: &ContractCallMetadata,
    cgas: &TypedExpression,
    bal: &TypedExpression,
    coin_color: &TypedExpression,
    user_argument: &TypedExpression,
    register_sequencer: &mut RegisterSequencer,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace,
    span: Span,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![];

    let user_argument_register = register_sequencer.next();
    let gas_to_forward = register_sequencer.next();
    let bal_register = register_sequencer.next();
    let coin_color_register = register_sequencer.next();
    let contract_address = register_sequencer.next();

    // load the function selector from the data section into a register
    let data_label =
        namespace.insert_data_value(&Literal::U32(u32::from_be_bytes(metadata.func_selector)));
    let selector_register = register_sequencer.next();
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::LWDataId(selector_register.clone(), data_label)),
        comment: "load fn selector for call".into(),
        owning_span: Some(span.clone()),
    });

    // evaluate the user provided argument to the contract
    asm_buf.append(&mut check!(
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

    // evaluate the gas to forward to the contract
    asm_buf.append(&mut check!(
        convert_expression_to_asm(cgas, namespace, &gas_to_forward, register_sequencer),
        vec![],
        warnings,
        errors
    ));

    // evaluate the balance to forward to the contract
    asm_buf.append(&mut check!(
        convert_expression_to_asm(bal, namespace, &bal_register, register_sequencer),
        vec![],
        warnings,
        errors
    ));

    // evaluate the coin color expression to forward to the contract
    asm_buf.append(&mut check!(
        convert_expression_to_asm(
            // investigation: changing this value also results in a different color
            coin_color,
            namespace,
            &coin_color_register,
            register_sequencer
        ),
        vec![],
        warnings,
        errors
    ));

    // evaluate the contract address for the contract
    asm_buf.append(&mut check!(
        convert_expression_to_asm(
            // investigation: changing the value in the contract_address register
            // impacts the color that the VM sees
            &*metadata.contract_address,
            namespace,
            &contract_address,
            register_sequencer
        ),
        vec![],
        warnings,
        errors
    ));

    // Write to memory, in order: the contract address (32 bytes), the function selector (param1, 8
    // bytes), and the user argument (param2, 8 bytes).
    //
    let ra_pointer = register_sequencer.next();
    // get the pointer to the beginning of free stack memory
    asm_buf.push(Op::unowned_register_move(
        ra_pointer.clone(),
        VirtualRegister::Constant(ConstantRegister::StackPointer),
    ));

    // extend the stack by 32 + 8 + 8 = 48 bytes
    asm_buf.push(Op::unowned_stack_allocate_memory(
        VirtualImmediate24::new_unchecked(
            48, // in bytes
            "constant infallible 48",
        ),
    ));

    // now $ra (ra_pointer) is pointing to the beginning of free stack memory, where we can write
    // the contract address and parameters
    //
    // first, copy the address over
    // write the contract addr to bytes 0-32
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::MCPI(
            ra_pointer.clone(),
            contract_address,
            VirtualImmediate12::new_unchecked(32, "infallible constant 32"),
        )),
        comment: "copy contract address for call".into(),
        owning_span: Some(span.clone()),
    });

    // write the selector to bytes 32-40
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::SW(
            ra_pointer.clone(),
            selector_register,
            // offset by 4 words, since a b256 is 4 words
            VirtualImmediate12::new_unchecked(4, "infallible constant 4"),
        )),
        comment: "write fn selector to rA + 32 for call".into(),
        owning_span: Some(span.clone()),
    });

    // write the user argument to bytes 40-48
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::SW(
            ra_pointer.clone(),
            user_argument_register,
            VirtualImmediate12::new_unchecked(5, "infallible constant 5"),
        )),
        comment: "move user param for call".into(),
        owning_span: Some(span.clone()),
    });

    // now, $rA (ra_pointer) points to the beginning of a section of contiguous memory that
    // contains the contract address, function selector, and user parameter.

    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::CALL(
            ra_pointer,
            bal_register,
            coin_color_register,
            gas_to_forward,
        )),
        comment: "call external contract".into(),
        owning_span: Some(span.clone()),
    });

    // now, move the return value of the contract call to the return register.
    // TODO validate RETL matches the expected type
    asm_buf.push(Op::register_move(
        return_register.into(),
        VirtualRegister::Constant(ConstantRegister::ReturnValue),
        span.clone(),
    ));

    ok(asm_buf, warnings, errors)
}
