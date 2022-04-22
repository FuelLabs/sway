use super::*;
use crate::{
    asm_lang::*,
    parse_tree::{BuiltinProperty, CallPath, Literal},
    semantic_analysis::{
        ast_node::{
            TypedAsmRegisterDeclaration, TypedCodeBlock, TypedEnumVariant, TypedExpressionVariant,
        },
        TypedExpression,
    },
    type_engine::*,
};
use sway_types::span::Span;

mod array;
mod contract_call;
mod enums;
mod if_exp;
mod lazy_op;
mod structs;
pub(crate) mod subfield;
use contract_call::convert_contract_call_to_asm;
use enums::convert_enum_instantiation_to_asm;
use if_exp::convert_if_exp_to_asm;
pub(crate) use structs::{
    convert_struct_expression_to_asm, convert_tuple_expression_to_asm, get_contiguous_memory_layout,
};
use subfield::convert_subfield_expression_to_asm;

/// Given a [TypedExpression], convert it to assembly and put its return value, if any, in the
/// `return_register`.
pub(crate) fn convert_expression_to_asm(
    exp: &TypedExpression,
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    match &exp.expression {
        TypedExpressionVariant::Literal(ref lit) => ok(
            convert_literal_to_asm(
                lit,
                namespace,
                return_register,
                register_sequencer,
                exp.span.clone(),
            ),
            warnings,
            errors,
        ),
        TypedExpressionVariant::FunctionApplication {
            name,
            contract_call_params,
            arguments,
            function_body,
            selector,
        } => {
            if let Some(metadata) = selector {
                convert_contract_call_to_asm(
                    metadata,
                    contract_call_params,
                    arguments,
                    register_sequencer,
                    return_register,
                    namespace,
                    exp.span.clone(),
                )
            } else {
                convert_fn_app_to_asm(
                    name,
                    arguments,
                    function_body,
                    namespace,
                    return_register,
                    register_sequencer,
                )
            }
        }
        TypedExpressionVariant::LazyOperator { op, lhs, rhs } => {
            lazy_op::convert_lazy_operator_to_asm(
                op,
                lhs,
                rhs,
                return_register,
                namespace,
                register_sequencer,
            )
        }
        TypedExpressionVariant::VariableExpression { name } => {
            let var = check!(
                namespace.look_up_variable(name),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(
                vec![Op::register_move(
                    return_register.into(),
                    var.into(),
                    exp.span.clone(),
                )],
                warnings,
                errors,
            )
        }
        TypedExpressionVariant::AsmExpression {
            registers,
            body,
            returns,
            whole_block_span,
        } => {
            let mut asm_buf = vec![];
            let mut warnings = vec![];
            let mut errors = vec![];
            // Keep track of the mapping from the declared names of the registers to the actual
            // registers from the sequencer for replacement
            let mut mapping_of_real_registers_to_declared_names: HashMap<&str, VirtualRegister> =
                Default::default();
            for TypedAsmRegisterDeclaration { name, initializer } in registers {
                let register = register_sequencer.next();
                assert_or_warn!(
                    ConstantRegister::parse_register_name(name.as_str()).is_none(),
                    warnings,
                    name.span().clone(),
                    Warning::ShadowingReservedRegister {
                        reg_name: name.clone()
                    }
                );

                mapping_of_real_registers_to_declared_names.insert(name.as_str(), register.clone());
                // evaluate each register's initializer
                if let Some(initializer) = initializer {
                    asm_buf.append(&mut check!(
                        convert_expression_to_asm(
                            initializer,
                            namespace,
                            &register,
                            register_sequencer,
                        ),
                        continue,
                        warnings,
                        errors
                    ));
                }
            }

            // For each opcode in the asm expression, attempt to parse it into an opcode and
            // replace references to the above registers with the newly allocated ones.
            for op in body {
                let replaced_registers = op.op_args.iter().map(|x| -> Result<_, CompileError> {
                    match realize_register(x.as_str(), &mapping_of_real_registers_to_declared_names)
                    {
                        Some(o) => Ok(o),
                        None => Err(CompileError::UnknownRegister {
                            span: x.span().clone(),
                            initialized_registers: mapping_of_real_registers_to_declared_names
                                .iter()
                                .map(|(name, _)| name.to_string())
                                .collect::<Vec<_>>()
                                .join("\n"),
                        }),
                    }
                });

                let replaced_registers = replaced_registers
                    .into_iter()
                    .filter_map(|x| match x {
                        Err(e) => {
                            errors.push(e);
                            None
                        }
                        Ok(o) => Some(o),
                    })
                    .collect::<Vec<VirtualRegister>>();

                // parse the actual op and registers
                let opcode = check!(
                    Op::parse_opcode(
                        &op.op_name,
                        replaced_registers.as_slice(),
                        &op.immediate,
                        op.span.clone()
                    ),
                    continue,
                    warnings,
                    errors
                );
                asm_buf.push(Op {
                    opcode: either::Either::Left(opcode),
                    comment: String::new(),
                    owning_span: Some(op.span.clone()),
                });
            }
            // Now, load the designated asm return register into the desired return register
            match (returns, return_register) {
                (Some((asm_reg, asm_reg_span)), return_reg) => {
                    // lookup and replace the return register
                    let mapped_asm_ret = match realize_register(
                        asm_reg.name.as_str(),
                        &mapping_of_real_registers_to_declared_names,
                    ) {
                        Some(reg) => reg,
                        None => {
                            errors.push(CompileError::UnknownRegister {
                                span: asm_reg_span.clone(),
                                initialized_registers: mapping_of_real_registers_to_declared_names
                                    .iter()
                                    .map(|(name, _)| name.to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            });
                            return err(warnings, errors);
                        }
                    };
                    asm_buf.push(Op::unowned_register_move_comment(
                        return_reg.clone(),
                        mapped_asm_ret,
                        "return value from inline asm",
                    ));
                }
                _ if look_up_type_id(exp.return_type).is_unit() => (),
                _ => {
                    errors.push(CompileError::InvalidAssemblyMismatchedReturn {
                        span: whole_block_span.clone(),
                    });
                }
            }
            ok(asm_buf, warnings, errors)
        }
        TypedExpressionVariant::StructExpression {
            struct_name,
            fields,
        } => convert_struct_expression_to_asm(
            struct_name,
            fields,
            return_register,
            namespace,
            register_sequencer,
        ),
        TypedExpressionVariant::StructFieldAccess {
            resolved_type_of_parent,
            prefix,
            field_to_access,
        } => convert_subfield_expression_to_asm(
            &exp.span,
            prefix,
            field_to_access.name.clone(),
            *resolved_type_of_parent,
            namespace,
            register_sequencer,
            return_register,
        ),
        // tuples are treated like mini structs, so we can use the same method that
        // struct field access uses
        TypedExpressionVariant::TupleElemAccess {
            resolved_type_of_parent,
            prefix,
            elem_to_access_span,
            elem_to_access_num,
        } => {
            // sorry
            let leaked_ix: &'static str = Box::leak(Box::new(elem_to_access_num.to_string()));
            let access_ident = Ident::new_with_override(leaked_ix, elem_to_access_span.clone());
            convert_subfield_expression_to_asm(
                &exp.span,
                prefix,
                access_ident,
                *resolved_type_of_parent,
                namespace,
                register_sequencer,
                return_register,
            )
        }
        TypedExpressionVariant::EnumInstantiation {
            enum_decl,
            tag,
            contents,
            instantiation_span,
            ..
        } => convert_enum_instantiation_to_asm(
            enum_decl,
            *tag,
            contents,
            return_register,
            namespace,
            register_sequencer,
            instantiation_span,
        ),
        TypedExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => convert_if_exp_to_asm(
            &**condition,
            &**then,
            r#else,
            return_register,
            namespace,
            register_sequencer,
        ),
        TypedExpressionVariant::CodeBlock(block) => {
            convert_code_block_to_asm(block, namespace, register_sequencer, Some(return_register))
        }
        TypedExpressionVariant::Array { contents } => array::convert_array_instantiation_to_asm(
            contents,
            namespace,
            return_register,
            register_sequencer,
        ),
        TypedExpressionVariant::ArrayIndex { prefix, index } => array::convert_array_index_to_asm(
            prefix,
            index,
            &exp.span,
            namespace,
            return_register,
            register_sequencer,
        ),
        TypedExpressionVariant::Tuple { fields } => {
            convert_tuple_expression_to_asm(fields, return_register, namespace, register_sequencer)
        }
        // ABI casts are purely compile-time constructs and generate no corresponding bytecode
        TypedExpressionVariant::AbiCast { .. } => ok(vec![], warnings, errors),
        TypedExpressionVariant::IfLet {
            enum_type,
            variant,
            then,
            r#else,
            variable_to_assign,
            expr,
        } => convert_if_let_to_asm(
            expr,
            *enum_type,
            variant,
            then,
            r#else,
            variable_to_assign,
            return_register,
            namespace,
            register_sequencer,
        ),
        TypedExpressionVariant::TypeProperty {
            property, type_id, ..
        } => match property {
            BuiltinProperty::SizeOfType => convert_size_of_to_asm(
                None,
                type_id,
                namespace,
                return_register,
                register_sequencer,
                exp.span.clone(),
            ),
            BuiltinProperty::IsRefType => convert_is_ref_type_to_asm(
                type_id,
                namespace,
                return_register,
                register_sequencer,
                exp.span.clone(),
            ),
        },
        TypedExpressionVariant::SizeOfValue { expr } => convert_size_of_to_asm(
            Some(expr),
            &expr.return_type,
            namespace,
            return_register,
            register_sequencer,
            exp.span.clone(),
        ),
        _ => {
            errors.push(CompileError::Unimplemented(
                "ASM generation has not yet been implemented for this.",
                exp.span.clone(),
            ));
            err(warnings, errors)
        }
    }
}

/// Takes a virtual register ID and either locates it in the register mapping, finds it is a reserved register,
/// or finds nothing and returns `None`.
fn realize_register(
    register_name: &str,
    mapping_of_real_registers_to_declared_names: &HashMap<&str, VirtualRegister>,
) -> Option<VirtualRegister> {
    match mapping_of_real_registers_to_declared_names.get(register_name) {
        Some(x) => Some(x.clone()),
        None => ConstantRegister::parse_register_name(register_name).map(VirtualRegister::Constant),
    }
}

pub(crate) fn convert_code_block_to_asm(
    block: &TypedCodeBlock,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    // Where to put the return value of this code block, if there was any.
    return_register: Option<&VirtualRegister>,
) -> CompileResult<Vec<Op>> {
    let mut asm_buf: Vec<Op> = vec![];
    let mut warnings = vec![];
    let mut errors = vec![];
    // generate a label for this block
    let exit_label = register_sequencer.get_label();
    for node in &block.contents {
        // If this is a return, then we jump to the end of the function and put the
        // value in the return register
        let res = check!(
            convert_node_to_asm(node, namespace, register_sequencer, return_register),
            continue,
            warnings,
            errors
        );
        match res {
            NodeAsmResult::JustAsm(ops) => asm_buf.append(&mut ops.into_iter().collect()),
            NodeAsmResult::ReturnStatement { mut asm } => {
                // insert a placeholder to jump to the end of the block and put the register
                asm_buf.append(&mut asm);
                asm_buf.push(Op::jump_to_label(exit_label.clone()));
            }
        }
    }
    asm_buf.push(Op::unowned_jump_label(exit_label));

    ok(asm_buf, warnings, errors)
}

/// Initializes [Literal] `lit` into [VirtualRegister] `return_register`.
fn convert_literal_to_asm(
    lit: &Literal,
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    _register_sequencer: &mut RegisterSequencer,
    span: Span,
) -> Vec<Op> {
    // first, insert the literal into the data section
    let data_id = namespace.insert_data_value(lit);
    // then get that literal id and use it to make a load word op
    vec![Op {
        opcode: either::Either::Left(VirtualOp::LWDataId(return_register.clone(), data_id)),
        comment: "literal instantiation".into(),
        owning_span: Some(span),
    }]
}

fn convert_is_ref_type_to_asm(
    type_id: &TypeId,
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
    span: Span,
) -> CompileResult<Vec<Op>> {
    let warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![Op::new_comment("is_ref_type".to_string())];
    let ty = match resolve_type(*type_id, &span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e.into());
            return err(warnings, errors);
        }
    };
    let is_ref_type = match ty.is_copy_type(&span) {
        Ok(is_copy) => !is_copy,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };
    let mut ops = convert_literal_to_asm(
        &Literal::Boolean(is_ref_type),
        namespace,
        return_register,
        register_sequencer,
        span,
    );
    asm_buf.append(&mut ops);
    ok(asm_buf, warnings, errors)
}

fn convert_size_of_to_asm(
    expr: Option<&TypedExpression>,
    type_id: &TypeId,
    namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
    span: Span,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![Op::new_comment("size_of_val".to_string())];
    if let Some(expr) = expr {
        let mut ops = check!(
            convert_expression_to_asm(expr, namespace, return_register, register_sequencer),
            vec![],
            warnings,
            errors
        );
        asm_buf.append(&mut ops);
    }
    let ty = match resolve_type(*type_id, &span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e.into());
            return err(warnings, errors);
        }
    };
    let size_in_bytes: u64 = match ty.size_in_bytes(&span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };
    let mut ops = convert_literal_to_asm(
        &Literal::U64(size_in_bytes),
        namespace,
        return_register,
        register_sequencer,
        span,
    );
    asm_buf.append(&mut ops);
    ok(asm_buf, warnings, errors)
}

/// For now, all functions are handled by inlining at the time of application.
fn convert_fn_app_to_asm(
    name: &CallPath,
    arguments: &[(Ident, TypedExpression)],
    function_body: &TypedCodeBlock,
    parent_namespace: &mut AsmNamespace,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![Op::new_comment(format!("{} fn call", name.suffix.as_str()))];
    // Make a local namespace so that the namespace of this function does not pollute the outer
    // scope
    let mut namespace = parent_namespace.clone();
    let mut args_and_registers: HashMap<Ident, VirtualRegister> = Default::default();
    // evaluate every expression being passed into the function
    for (name, arg) in arguments {
        let return_register = register_sequencer.next();
        let mut ops = check!(
            convert_expression_to_asm(arg, &mut namespace, &return_register, register_sequencer),
            vec![],
            warnings,
            errors
        );
        asm_buf.append(&mut ops);
        args_and_registers.insert(name.clone(), return_register);
    }

    // insert the arguments into the asm namespace with their registers mapped
    for (name, reg) in args_and_registers {
        namespace.insert_variable(name, reg);
    }

    // evaluate the function body
    let mut body = check!(
        convert_code_block_to_asm(
            function_body,
            &mut namespace,
            register_sequencer,
            Some(return_register),
        ),
        vec![],
        warnings,
        errors
    );
    asm_buf.append(&mut body);
    parent_namespace.data_section = namespace.data_section;

    // the return  value is already put in its proper register via the above statement, so the buf
    // is done
    ok(asm_buf, warnings, errors)
}

/// This is similar to `convert_fn_app_to_asm()`, except instead of function arguments, this takes
/// a list of registers corresponding to the arguments and three additional registers corresponding
/// to the contract call parameters (gas, coins, asset_id).  
///
/// All registers are expected to be
/// pre-loaded with the desired values when this function is jumped to.
///
pub(crate) fn convert_abi_fn_to_asm(
    decl: &TypedFunctionDeclaration,
    arguments: &[(Ident, VirtualRegister)],
    parent_namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![Op::new_comment(format!("{} abi fn", decl.name.as_str()))];
    // Make a local namespace so that the namespace of this function does not pollute the outer
    // scope
    let mut namespace = parent_namespace.clone();
    let return_register = register_sequencer.next();

    // insert the arguments into the asm namespace with their registers mapped
    for arg in arguments {
        namespace.insert_variable(arg.clone().0, arg.clone().1);
    }
    // evaluate the function body
    let mut body = check!(
        convert_code_block_to_asm(
            &decl.body,
            &mut namespace,
            register_sequencer,
            Some(&return_register),
        ),
        vec![],
        warnings,
        errors
    );

    asm_buf.append(&mut body);
    // return the value from the abi function
    asm_buf.append(&mut check!(
        ret_or_retd_value(decl, return_register, register_sequencer, &mut namespace),
        return err(warnings, errors),
        warnings,
        errors
    ));

    parent_namespace.data_section = namespace.data_section;

    // the return  value is already put in its proper register via the above statement, so the buf
    // is done
    ok(asm_buf, warnings, errors)
}

#[allow(clippy::too_many_arguments)]
fn convert_if_let_to_asm(
    expr: &TypedExpression,
    _enum_type: TypeId,
    variant: &TypedEnumVariant,
    then: &TypedCodeBlock,
    r#else: &Option<Box<TypedExpression>>,
    variable_to_assign: &Ident,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    // 1. evaluate the expression
    // 2. load the expected tag into a register ($rA)
    // 3. compare the tag to the first word of the expression's returned value
    // 4. grab a register for `variable_to_assign`, insert it into the asm namespace
    // 5. if the tags are equal, load the returned value from byte 1..end into `variable_to_assign`
    // 5.5 if they are not equal, jump to the label in 7
    // 6. evaluate the then branch with that variable in scope
    // 7. insert a jump label for the else branch
    // 8. evaluate the else branch, if any
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut buf = vec![];
    // 1.
    let expr_return_register = register_sequencer.next();
    let mut expr_buf = check!(
        convert_expression_to_asm(&*expr, namespace, &expr_return_register, register_sequencer),
        vec![],
        warnings,
        errors
    );
    buf.append(&mut expr_buf);
    // load the tag from the evaluated value
    // as this is an enum we know the value in the register is a pointer
    // we can therefore read a word from the register and move it into another register
    let received_tag_register = register_sequencer.next();
    buf.push(Op {
        opcode: Either::Left(VirtualOp::LW(
            received_tag_register.clone(),
            expr_return_register.clone(),
            VirtualImmediate12::new_unchecked(0, "infallible"),
        )),
        comment: "load received enum tag".into(),
        owning_span: Some(expr.span.clone()),
    });
    // 2.
    let expected_tag_register = register_sequencer.next();
    let expected_tag_label = namespace.insert_data_value(&Literal::U64(variant.tag as u64));
    buf.push(Op {
        opcode: either::Either::Left(VirtualOp::LWDataId(
            expected_tag_register.clone(),
            expected_tag_label,
        )),
        comment: "load enum tag for if let".into(),
        owning_span: Some(expr.span.clone()),
    });
    let label_for_else_branch = register_sequencer.get_label();
    // 3 - 5
    buf.push(Op {
        opcode: Either::Right(OrganizationalOp::JumpIfNotEq(
            expected_tag_register,
            received_tag_register,
            label_for_else_branch.clone(),
        )),
        comment: "jump to if let's else branch".into(),
        owning_span: Some(expr.span.clone()),
    });
    // 6.
    // put the destructured variable into the namespace for the then branch, but not otherwise
    let mut then_branch_asm_namespace = namespace.clone();
    let variable_to_assign_register = register_sequencer.next();
    then_branch_asm_namespace.insert_variable(
        variable_to_assign.clone(),
        variable_to_assign_register.clone(),
    );
    // load the word that is at the expr return register + 1 word
    // + 1 word is to account for the enum tag
    buf.push(Op {
        opcode: Either::Left(VirtualOp::LW(
            variable_to_assign_register,
            expr_return_register,
            VirtualImmediate12::new_unchecked(1, "infallible"),
        )),
        owning_span: Some(then.span().clone()),
        comment: "Load destructured value into register".into(),
    });

    // 6
    buf.append(&mut check!(
        convert_code_block_to_asm(
            then,
            &mut then_branch_asm_namespace,
            register_sequencer,
            Some(return_register)
        ),
        return err(warnings, errors),
        warnings,
        errors
    ));

    // add the data section from the then branch back to the main one

    namespace.overwrite_data_section(then_branch_asm_namespace);

    let label_for_after_else_branch = register_sequencer.get_label();
    if let Some(r#else) = r#else {
        buf.push(Op::jump_to_label_comment(
            label_for_after_else_branch.clone(),
            "jump to after the else branch",
        ));

        buf.push(Op::unowned_jump_label(label_for_else_branch));

        buf.append(&mut check!(
            convert_expression_to_asm(r#else, namespace, return_register, register_sequencer),
            return err(warnings, errors),
            warnings,
            errors
        ));

        buf.push(Op::unowned_jump_label(label_for_after_else_branch));
    }

    ok(buf, warnings, errors)
}
