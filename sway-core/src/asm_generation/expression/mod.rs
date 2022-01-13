use super::*;
use crate::{
    asm_lang::*,
    parse_tree::{CallPath, Literal},
    semantic_analysis::{
        ast_node::{TypedAsmRegisterDeclaration, TypedCodeBlock, TypedExpressionVariant},
        TypedExpression,
    },
    type_engine::look_up_type_id,
};
use sway_types::span::Span;

mod array;
mod contract_call;
mod enums;
mod if_exp;
mod lazy_op;
mod structs;
mod subfield;
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
            arguments,
            function_body,
            selector,
        } => {
            if let Some(metadata) = selector {
                assert_eq!(
                    arguments.len(),
                    4,
                    "this is verified in the semantic analysis stage"
                );
                convert_contract_call_to_asm(
                    metadata,
                    // gas to forward
                    &arguments[0].1,
                    // coins to forward
                    &arguments[1].1,
                    // color of coins
                    &arguments[2].1,
                    // user parameter
                    &arguments[3].1,
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
                /*
                errors.append(
                    &mut op
                        .op_args
                        .iter()
                        .filter_map(|Ident { primary_name, span }| {
                            if mapping_of_real_registers_to_declared_names
                                .get(primary_name)
                                .is_none() &&
                            {
                                Some(todo!("error! {:?}", primary_name))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                */
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
            field_to_access_span,
        } => convert_subfield_expression_to_asm(
            &exp.span,
            prefix,
            &field_to_access.name,
            field_to_access_span.clone(),
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
            elem_to_access_num,
            elem_to_access_span,
        } => convert_subfield_expression_to_asm(
            &exp.span,
            prefix,
            &format!("{}", elem_to_access_num),
            elem_to_access_span.clone(),
            *resolved_type_of_parent,
            namespace,
            register_sequencer,
            return_register,
        ),
        /*
        TypedExpressionVariant::EnumArgAccess {
            prefix,
            variant_to_access,
            arg_num_to_access,
            resolved_type_of_parent,
        } => convert_enum_arg_expression_to_asm(
            &exp.span,
            prefix,
            variant_to_access,
            arg_num_to_access.to_owned(),
            *resolved_type_of_parent,
            namespace,
            register_sequencer,
            return_register,
        ),
        */
        TypedExpressionVariant::EnumInstantiation {
            enum_decl,
            variant_name,
            tag,
            contents,
        } => convert_enum_instantiation_to_asm(
            enum_decl,
            variant_name,
            *tag,
            contents,
            return_register,
            namespace,
            register_sequencer,
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
        a => {
            println!("unimplemented: {:?}", a);
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

/// This is similar to `convert_fn_app_to_asm()`, except instead of function arguments, this
/// takes four registers where the registers are expected to be pre-loaded with the desired values
/// when this function is jumped to.
pub(crate) fn convert_abi_fn_to_asm(
    decl: &TypedFunctionDeclaration,
    user_argument: (Ident, VirtualRegister),
    cgas: (Ident, VirtualRegister),
    bal: (Ident, VirtualRegister),
    coin_color: (Ident, VirtualRegister),
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
    namespace.insert_variable(user_argument.0, user_argument.1);
    namespace.insert_variable(cgas.0, cgas.1);
    namespace.insert_variable(bal.0, bal.1);
    namespace.insert_variable(coin_color.0, coin_color.1);
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
