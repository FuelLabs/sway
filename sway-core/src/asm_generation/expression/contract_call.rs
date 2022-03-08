use super::*;
use crate::{constants, semantic_analysis::ast_node::*};
use either::Either;

/// Converts a function application of a contract ABI function into assembly
#[allow(clippy::too_many_arguments)]
pub(crate) fn convert_contract_call_to_asm(
    metadata: &ContractCallMetadata,
    contract_call_parameters: &HashMap<String, TypedExpression>,
    arguments: &[(Ident, TypedExpression)],
    register_sequencer: &mut RegisterSequencer,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace,
    span: Span,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![];

    let bundled_arguments_register = register_sequencer.next();
    let gas_register = register_sequencer.next();
    let coins_register = register_sequencer.next();
    let asset_id_register = register_sequencer.next();
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

    // Bundle the arguments into a struct if needed. If we only have a single argument, there is no
    // need to bundle.
    let bundled_arguments = match arguments.len() {
        0 => None,
        1 => Some(arguments[0].1.clone()),
        _ => {
            // create a struct expression that bundles the arguments in order
            let mut typed_fields_buf = vec![];
            for (name, arg) in arguments {
                typed_fields_buf.push(TypedStructExpressionField {
                    value: arg.clone(),
                    name: name.clone(),
                });
            }
            Some(TypedExpression {
                expression: TypedExpressionVariant::StructExpression {
                    struct_name: Ident::new_with_override("bundled_arguments", span.clone()),
                    fields: typed_fields_buf,
                },
                return_type: 0,
                is_constant: IsConstant::No,
                span: span.clone(),
            })
        }
    };

    // evaluate the bundle of arguments
    if let Some(bundled_arguments) = &bundled_arguments {
        asm_buf.append(&mut check!(
            convert_expression_to_asm(
                bundled_arguments,
                namespace,
                &bundled_arguments_register,
                register_sequencer
            ),
            vec![],
            warnings,
            errors
        ));
    }

    // evaluate the gas to forward to the contract. If no user-specified gas parameter is found,
    // simply load $cgas.
    match contract_call_parameters.get(&constants::CONTRACT_CALL_GAS_PARAMETER_NAME.to_string()) {
        Some(exp) => asm_buf.append(&mut check!(
            convert_expression_to_asm(exp, namespace, &gas_register, register_sequencer),
            vec![],
            warnings,
            errors
        )),
        None => asm_buf.push(load_gas(gas_register.clone())),
    }

    // evaluate the coins balance to forward to the contract. If no user-specified coins parameter
    // is found, simply load $bal.
    match contract_call_parameters.get(&constants::CONTRACT_CALL_COINS_PARAMETER_NAME.to_string()) {
        Some(exp) => asm_buf.append(&mut check!(
            convert_expression_to_asm(exp, namespace, &coins_register, register_sequencer),
            vec![],
            warnings,
            errors
        )),
        None => {
            let coins_default = namespace.insert_data_value(&Literal::U64(
                constants::CONTRACT_CALL_COINS_PARAMETER_DEFAULT_VALUE,
            ));
            asm_buf.push(Op {
                opcode: Either::Left(VirtualOp::LWDataId(coins_register.clone(), coins_default)),
                owning_span: None,
                comment: "loading the default coins value for call".into(),
            })
        }
    }

    // evaluate the asset_id expression to forward to the contract. If no user-specified asset_id parameter
    // is found, simply load $fp.
    match contract_call_parameters
        .get(&constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME.to_string())
    {
        Some(exp) => asm_buf.append(&mut check!(
            convert_expression_to_asm(exp, namespace, &asset_id_register, register_sequencer),
            vec![],
            warnings,
            errors
        )),
        None => {
            let asset_id_default = namespace.insert_data_value(&Literal::B256(
                constants::CONTRACT_CALL_ASSET_ID_PARAMETER_DEFAULT_VALUE,
            ));
            asm_buf.push(Op {
                opcode: Either::Left(VirtualOp::LWDataId(
                    asset_id_register.clone(),
                    asset_id_default,
                )),
                owning_span: None,
                comment: "loading the default asset_id value for call".into(),
            })
        }
    }

    // evaluate the contract address for the contract
    asm_buf.append(&mut check!(
        convert_expression_to_asm(
            // investigation: changing the value in the contract_address register
            // impacts the asset_id that the VM sees
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
    if bundled_arguments.is_some() {
        asm_buf.push(Op {
            opcode: Either::Left(VirtualOp::SW(
                ra_pointer.clone(),
                bundled_arguments_register,
                VirtualImmediate12::new_unchecked(5, "infallible constant 5"),
            )),
            comment: "move user param for call".into(),
            owning_span: Some(span.clone()),
        });
    }

    // now, $rA (ra_pointer) points to the beginning of a section of contiguous memory that
    // contains the contract address, function selector, and user parameter.

    asm_buf.push(Op {
        opcode: Either::Left(VirtualOp::CALL(
            ra_pointer,
            coins_register,
            asset_id_register,
            gas_register,
        )),
        comment: "call external contract".into(),
        owning_span: Some(span.clone()),
    });

    // now, move the return value of the contract call to the return register.
    // TODO validate RETL matches the expected type
    asm_buf.push(Op::register_move(
        return_register.into(),
        VirtualRegister::Constant(ConstantRegister::ReturnValue),
        span,
    ));

    ok(asm_buf, warnings, errors)
}
