use super::compiler_constants::{TWELVE_BITS, TWENTY_FOUR_BITS};
use super::*;

pub(super) fn convert_array_instantiation_to_asm(
    contents: &[TypedExpression],
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let warnings = Vec::new();
    let mut errors = Vec::new();

    // Even for empty arrays we need to set the return register to something.
    let mut bytecode = vec![Op::unowned_register_move(
        return_register.clone(),
        VirtualRegister::Constant(ConstantRegister::StackPointer),
    )];

    // If the array is empty then this is a NOOP.  The array will point to zero elements at SP.
    if contents.is_empty() {
        return ok(bytecode, warnings, errors);
    }

    // Get the array element size.
    let elem_type = check_std_result!(
        resolve_type(contents[0].return_type, &contents[0].span),
        warnings,
        errors
    );
    let elem_size_in_words =
        check_std_result!(elem_type.size_in_words(&contents[0].span), warnings, errors);
    let mut array_size = elem_size_in_words * 8 * contents.len() as u64;

    // Reserve space on the stack.  We may need more than one expansion to cover the entire array.
    while array_size != 0 {
        let expansion_size = std::cmp::min(TWENTY_FOUR_BITS, array_size);
        bytecode.push(Op::unowned_stack_allocate_memory(
            VirtualImmediate24::new_unchecked(expansion_size, "guaranteed to be < than 2^24"),
        ));
        array_size -= expansion_size;
    }

    // Initialise each array element in turn.  Ideally they'd be initialised in place, but that can
    // wait until we have IR.
    //
    // If the element doesn't fit in a single register (i.e., we need to use MCP) or the array is
    // large enough that the last element offset can't fit in 12 bits then we need to use a
    // register to track that offset.
    if elem_size_in_words > 1 || (contents.len() as u64 - 1) > TWELVE_BITS {
        initialize_large_array_instantiation(
            contents,
            elem_size_in_words,
            return_register,
            bytecode,
            namespace,
            register_sequencer,
            warnings,
            errors,
        )
    } else {
        initialize_small_array_instantiation(
            contents,
            return_register,
            bytecode,
            namespace,
            register_sequencer,
            warnings,
            errors,
        )
    }
}

// Initialise an array with an element size in words of 1 and where all elements can be addressed
// with twelve bits.
fn initialize_small_array_instantiation(
    contents: &[TypedExpression],
    array_start_reg: &VirtualRegister,
    mut bytecode: Vec<Op>,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    mut warnings: Vec<CompileWarning>,
    mut errors: Vec<CompileError>,
) -> CompileResult<Vec<Op>> {
    assert!(contents.len() as u64 - 1 <= TWELVE_BITS);

    let elem_init_reg = register_sequencer.next();
    for (idx, elem) in contents.iter().enumerate() {
        bytecode.append(&mut check!(
            convert_expression_to_asm(elem, namespace, &elem_init_reg.clone(), register_sequencer),
            Vec::new(),
            warnings,
            errors
        ));

        bytecode.push(Op::write_register_to_memory(
            array_start_reg.clone(),
            elem_init_reg.clone(),
            VirtualImmediate12::new_unchecked(idx as u64, "array is indexable with 12 bits"),
            elem.span.clone(),
        ));
    }

    ok(bytecode, warnings, errors)
}

#[allow(clippy::too_many_arguments)]
fn initialize_large_array_instantiation(
    contents: &[TypedExpression],
    elem_size_in_words: u64,
    array_offs_reg: &VirtualRegister,
    mut bytecode: Vec<Op>,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    mut warnings: Vec<CompileWarning>,
    mut errors: Vec<CompileError>,
) -> CompileResult<Vec<Op>> {
    let elem_offs_reg = register_sequencer.next();
    bytecode.push(Op::unowned_register_move(
        elem_offs_reg.clone(),
        array_offs_reg.clone(),
    ));

    let elem_init_reg = register_sequencer.next();
    for elem in contents {
        bytecode.append(&mut check!(
            convert_expression_to_asm(elem, namespace, &elem_init_reg.clone(), register_sequencer),
            Vec::new(),
            warnings,
            errors
        ));

        if elem_size_in_words == 1 {
            // Elem size is 1 then elem_init_reg is the value itself and we can use SW.
            bytecode.push(Op::write_register_to_memory(
                elem_offs_reg.clone(),
                elem_init_reg.clone(),
                VirtualImmediate12 { value: 0 },
                elem.span.clone(),
            ));
            bytecode.push(Op {
                opcode: either::Either::Left(VirtualOp::ADDI(
                    elem_offs_reg.clone(),
                    elem_offs_reg.clone(),
                    VirtualImmediate12 { value: 8 },
                )),
                owning_span: Some(elem.span.clone()),
                comment: "increment to next element offset".into(),
            });
        } else {
            // Elem size is > 1, so elem_init_reg is a pointer.
            //
            // If elem size _in bytes_ doesn't fit in 12 bits then we need to do multiple copies
            // per element.
            let mut elem_size_in_bytes = elem_size_in_words * 8;
            while elem_size_in_bytes != 0 {
                let copy_size = std::cmp::min(TWELVE_BITS, elem_size_in_bytes);

                bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::MCPI(
                        elem_offs_reg.clone(),
                        elem_init_reg.clone(),
                        VirtualImmediate12::new_unchecked(
                            copy_size,
                            "guaranteed to be < than 2^12",
                        ),
                    )),
                    owning_span: Some(elem.span.clone()),
                    comment: format!("cp array element size {}", copy_size),
                });
                bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::ADDI(
                        elem_offs_reg.clone(),
                        elem_offs_reg.clone(),
                        VirtualImmediate12::new_unchecked(
                            copy_size,
                            "guaranteed to be < than 2^12",
                        ),
                    )),
                    owning_span: Some(elem.span.clone()),
                    comment: "increment to next element offset".into(),
                });
                bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::ADDI(
                        elem_init_reg.clone(),
                        elem_init_reg.clone(),
                        VirtualImmediate12::new_unchecked(
                            copy_size,
                            "guaranteed to be < than 2^12",
                        ),
                    )),
                    owning_span: Some(elem.span.clone()),
                    comment: "increment to next init offset".into(),
                });

                elem_size_in_bytes -= copy_size;
            }
        }
    }

    ok(bytecode, warnings, errors)
}

pub(super) fn convert_array_index_to_asm(
    prefix: &TypedExpression,
    index: &TypedExpression,
    span: &Span,
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut bytecode = Vec::new();

    // Get the array element type and count.
    let (elem_type, count) = match check_std_result!(
        resolve_type(prefix.return_type, &prefix.span),
        warnings,
        errors
    ) {
        TypeInfo::Array(elem_type_id, count) => (
            check_std_result!(resolve_type(elem_type_id, &prefix.span), warnings, errors),
            count as u64,
        ),
        _otherwise => {
            errors.push(CompileError::Internal(
                "attempt to index a non-array",
                span.clone(),
            ));
            return err(warnings, errors);
        }
    };

    // Check for out of bounds if we have a literal index.
    if let TypedExpressionVariant::Literal(Literal::U64(index)) = index.expression {
        if index >= count {
            errors.push(CompileError::ArrayOutOfBounds {
                index,
                count,
                span: span.clone(),
            });
            return err(warnings, errors);
        }
    }

    let prefix_reg = register_sequencer.next();
    bytecode.append(&mut check!(
        convert_expression_to_asm(prefix, namespace, &prefix_reg, register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    ));

    let index_reg = register_sequencer.next();
    bytecode.append(&mut check!(
        convert_expression_to_asm(index, namespace, &index_reg.clone(), register_sequencer),
        return err(warnings, errors),
        warnings,
        errors
    ));

    // Put the last valid array index (count - 1) into a register using a recursive helper.
    let count_reg = register_sequencer.next();
    set_large_register_value(count - 1, &count_reg, &mut bytecode, span);

    // Add an assertion that the index is within bounds.  When we have IR we can more easily check
    // if the index is a) constant, and b) within bounds, removing the need for a runtime
    // assertion.
    compile_bounds_assertion(
        &mut bytecode,
        &count_reg,
        &index_reg,
        span,
        register_sequencer,
    );

    // Get the element size in words first.
    let elem_size_in_words =
        check_std_result!(elem_type.size_in_words(&prefix.span), warnings, errors);

    // The element offset can be calculated as a byte offset.  We need to multiply the index by the
    // element size.
    let elem_size_reg = register_sequencer.next();
    set_large_register_value(elem_size_in_words * 8, &elem_size_reg, &mut bytecode, span);

    let elem_offs_reg = register_sequencer.next();
    bytecode.push(Op {
        opcode: either::Either::Left(VirtualOp::MUL(
            elem_offs_reg.clone(),
            index_reg.clone(),
            elem_size_reg.clone(),
        )),
        owning_span: Some(span.clone()),
        comment: "convert index into byte offset".into(),
    });
    bytecode.push(Op {
        opcode: either::Either::Left(VirtualOp::ADD(
            elem_offs_reg.clone(),
            prefix_reg,
            elem_offs_reg.clone(),
        )),
        owning_span: Some(span.clone()),
        comment: "add element offset to array base offset".into(),
    });

    // If the element size is 1 then we fetch it with LW, otherwise we return the pointer.
    if elem_size_in_words == 1 {
        //bytecode.push(Op {
        //    opcode: either::Either::Left(VirtualOp::LOG(
        //        prefix_reg.clone(),
        //        index_reg.clone(),
        //        elem_offs_reg.clone(),
        //        VirtualRegister::Constant(ConstantRegister::Zero),
        //    )),
        //    owning_span: None,
        //    comment: "prefix, index, offset, 0".into(),
        //});
        bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::LW(
                return_register.clone(),
                elem_offs_reg,
                VirtualImmediate12 { value: 0 },
            )),
            owning_span: Some(span.clone()),
            comment: "load array element".into(),
        });
    } else {
        bytecode.push(Op::unowned_register_move(
            return_register.clone(),
            elem_offs_reg,
        ));
    }

    ok(bytecode, warnings, errors)
}

// Recursively put a value into a regiser 12 bits at a time using OR and SLL.
//
// We want the first (and usually probably the only) operation to OR with Zero, so we recurse for
// each set of 12 bits until we hit a zero value, and then return the Zero register to be used
// next.  Thereafter we OR the destination register.
fn set_large_register_value<'a>(
    value: u64,
    dst_reg: &'a VirtualRegister,
    bytecode: &mut Vec<Op>,
    span: &Span,
) -> &'a VirtualRegister {
    if value == 0 {
        return &VirtualRegister::Constant(ConstantRegister::Zero);
    }

    // Value is non-zero; fill in higher bits first.
    let src_reg = set_large_register_value(value >> 12, dst_reg, bytecode, span);
    if value > TWELVE_BITS {
        bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::SLLI(
                src_reg.clone(),
                src_reg.clone(),
                VirtualImmediate12 { value: 12 },
            )),
            owning_span: Some(span.clone()),
            comment: "shift high bits of value".into(),
        });
    }

    // Fill in the lower bits.
    bytecode.push(Op {
        opcode: either::Either::Left(VirtualOp::ORI(
            dst_reg.clone(),
            src_reg.clone(),
            VirtualImmediate12::new_unchecked(value & TWELVE_BITS, "guaranteed to be < than 2^12"),
        )),
        owning_span: Some(span.clone()),
        comment: "setting value bits".into(),
    });
    dst_reg
}

fn compile_bounds_assertion(
    bytecode: &mut Vec<Op>,
    count_reg: &VirtualRegister,
    index_reg: &VirtualRegister,
    span: &Span,
    register_sequencer: &mut RegisterSequencer,
) {
    // gt_reg = index_reg > count_reg.
    let gt_reg = register_sequencer.next();
    bytecode.push(Op {
        opcode: either::Either::Left(VirtualOp::GT(
            gt_reg.clone(),
            index_reg.clone(),
            count_reg.clone(),
        )),
        owning_span: Some(span.clone()),
        comment: "compare array index for out of bounds".into(),
    });

    // Jump past the RVRT if gt_reg is 0.
    let skip_label = register_sequencer.get_label();
    bytecode.push(Op::jump_if_not_equal(
        gt_reg,
        VirtualRegister::Constant(ConstantRegister::One),
        skip_label.clone(),
    ));

    // Revert.
    bytecode.push(Op {
        opcode: either::Either::Left(VirtualOp::RVRT(VirtualRegister::Constant(
            ConstantRegister::One,
        ))),
        owning_span: Some(span.clone()),
        comment: "aborting due to out of bounds access".into(),
    });

    bytecode.push(Op::jump_label_comment(
        skip_label,
        span.clone(),
        "after bounds check",
    ));
}
