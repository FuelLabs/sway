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
    asm_buf.append(&mut type_check!(
        convert_expression_to_asm(coin_color, namespace, &contract_address, register_sequencer),
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
    //
    //  load 32 into a register
    let data_label = namespace.insert_data_value(&Literal::U32(32));
    let num32_register = register_sequencer.next();
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::LWDataId(num32_register.clone(), data_label)),
        comment: "constant 32 load for call".into(),
        owning_span: Some(span.clone()),
    });
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::MCP(
            ra_pointer.clone(),
            contract_address,
            num32_register,
        )),
        comment: "move contract address for call".into(),
        owning_span: Some(span.clone()),
    });
    // second, calculate the new pointer (current value of $rA + 32)
    let rover_register = register_sequencer.next();
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::ADDI(
            rover_register.clone(),
            ra_pointer.clone(),
            VirtualImmediate12::new_unchecked(32, "infallible constant 32"),
        )),
        comment: "calculate call fn selector addr".into(),
        owning_span: Some(span.clone()),
    });
    // third, use the rover register as the pointer to write the function selector to
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::MCP(
            rover_register.clone(),
            selector_register,
            VirtualRegister::Constant(ConstantRegister::One),
        )),
        comment: "move fn selector for call".into(),
        owning_span: Some(span.clone()),
    });
    // fourth, calculate the new pointer (current value of the rover register + 1)
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::ADDI(
            rover_register.clone(),
            rover_register.clone(),
            VirtualImmediate12::new_unchecked(1, "infallible constant 1"),
        )),
        comment: "calculate call user param addr".into(),
        owning_span: Some(span.clone()),
    });
    // fifth, use the rover register as the pointer to write the user param to
    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::MCP(
            rover_register,
            user_argument_register,
            VirtualRegister::Constant(ConstantRegister::One),
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
        owning_span: Some(span),
    });

    ok(asm_buf, warnings, errors)
}
