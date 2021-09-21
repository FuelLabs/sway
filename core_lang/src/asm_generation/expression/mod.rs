use super::*;
use crate::span::Span;
use crate::{
    asm_lang::{virtual_ops::VirtualRegister, *},
    parse_tree::CallPath,
};
use crate::{
    parse_tree::Literal,
    semantic_analysis::{
        ast_node::{TypedAsmRegisterDeclaration, TypedCodeBlock, TypedExpressionVariant},
        TypedExpression,
    },
    types::{MaybeResolvedType, ResolvedType},
};

mod contract_call;
mod enum_instantiation;
mod if_exp;
mod structs;
mod subfield;
use contract_call::convert_contract_call_to_asm;
use enum_instantiation::convert_enum_instantiation_to_asm;
use if_exp::convert_if_exp_to_asm;
pub(crate) use structs::{convert_struct_expression_to_asm, get_struct_memory_layout};
use subfield::convert_subfield_expression_to_asm;

/// Given a [TypedExpression], convert it to assembly and put its return value, if any, in the
/// `return_register`.
pub(crate) fn convert_expression_to_asm<'sc>(
    exp: &TypedExpression<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
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
            for TypedAsmRegisterDeclaration {
                name,
                initializer,
                name_span,
            } in registers
            {
                let register = register_sequencer.next();
                assert_or_warn!(
                    ConstantRegister::parse_register_name(name).is_none(),
                    warnings,
                    name_span.clone(),
                    Warning::ShadowingReservedRegister { reg_name: &name }
                );

                mapping_of_real_registers_to_declared_names.insert(name, register.clone());
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
                let replaced_registers = op
                    .op_args
                    .iter()
                    .map(|x| -> Result<_, CompileError> {
                        match realize_register(
                            x.primary_name,
                            &mapping_of_real_registers_to_declared_names,
                        ) {
                            Some(o) => Ok(o),
                            None => Err(CompileError::UnknownRegister {
                                span: x.span.clone(),
                                initialized_registers: mapping_of_real_registers_to_declared_names
                                    .iter()
                                    .map(|(name, _)| name.to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            }),
                        }
                    })
                    .collect::<Vec<Result<_, _>>>();

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
                        mapped_asm_ret.clone(),
                        "return value from inline asm",
                    ));
                }
                _ if exp.return_type == MaybeResolvedType::Resolved(ResolvedType::Unit) => (),
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
        } => convert_struct_expression_to_asm(struct_name, fields, namespace, register_sequencer),
        TypedExpressionVariant::StructFieldAccess {
            resolved_type_of_parent,
            prefix,
            field_to_access,
        } => convert_subfield_expression_to_asm(
            &exp.span,
            prefix,
            field_to_access,
            resolved_type_of_parent,
            namespace,
            register_sequencer,
            return_register,
        ),
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
        TypedExpressionVariant::Unit => ok(vec![], warnings, errors),
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
        None => match ConstantRegister::parse_register_name(register_name) {
            Some(x) => Some(VirtualRegister::Constant(x)),
            None => None,
        },
    }
}

pub(crate) fn convert_code_block_to_asm<'sc>(
    block: &TypedCodeBlock<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
    // Where to put the return value of this code block, if there was any.
    return_register: Option<&VirtualRegister>,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
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
fn convert_literal_to_asm<'sc>(
    lit: &Literal<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    return_register: &VirtualRegister,
    _register_sequencer: &mut RegisterSequencer,
    span: Span<'sc>,
) -> Vec<Op<'sc>> {
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
fn convert_fn_app_to_asm<'sc>(
    name: &CallPath<'sc>,
    arguments: &[(Ident<'sc>, TypedExpression<'sc>)],
    function_body: &TypedCodeBlock<'sc>,
    parent_namespace: &mut AsmNamespace<'sc>,
    return_register: &VirtualRegister,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![Op::new_comment(format!(
        "{} fn call",
        name.suffix.primary_name
    ))];
    // Make a local namespace so that the namespace of this function does not pollute the outer
    // scope
    let mut namespace = parent_namespace.clone();
    let mut args_and_registers: HashMap<Ident<'sc>, VirtualRegister> = Default::default();
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
pub(crate) fn convert_abi_fn_to_asm<'sc>(
    decl: &TypedFunctionDeclaration<'sc>,
    user_argument: (Ident<'sc>, VirtualRegister),
    cgas: (Ident<'sc>, VirtualRegister),
    bal: (Ident<'sc>, VirtualRegister),
    coin_color: (Ident<'sc>, VirtualRegister),
    parent_namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![Op::new_comment(format!(
        "{} abi fn",
        decl.name.primary_name
    ))];
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
        ret_or_retd_value(&decl, return_register, register_sequencer, &mut namespace),
        return err(warnings, errors),
        warnings,
        errors
    ));

    parent_namespace.data_section = namespace.data_section;

    // the return  value is already put in its proper register via the above statement, so the buf
    // is done
    ok(asm_buf, warnings, errors)
}
