use super::*;
use crate::{parse_tree::CallPath, vendored_vm::*};
use crate::{
    parse_tree::Literal,
    semantics::{
        ast_node::{TypedCodeBlock, TypedExpressionVariant},
        TypedExpression,
    },
};
use either::Either;
use pest::Span;

pub(crate) enum ExpressionAsmResult<'sc> {
    Ops(Vec<Op<'sc>>),
    Shortcut(AsmRegister),
}

/// Given a [TypedExpression], convert it to assembly and put its return value, if any, in the
/// `return_register`.
pub(crate) fn convert_expression_to_asm<'sc>(
    exp: &TypedExpression<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    return_register: &AsmRegister,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    match &exp.expression {
        TypedExpressionVariant::Literal(ref lit) => convert_literal_to_asm(
            lit,
            namespace,
            return_register,
            register_sequencer,
            exp.span.clone(),
        ),
        TypedExpressionVariant::FunctionApplication {
            name,
            arguments,
            function_body,
        } => convert_fn_app_to_asm(
            name,
            arguments,
            function_body,
            namespace,
            return_register,
            register_sequencer,
        ),
        TypedExpressionVariant::VariableExpression { unary_op, name } => {
            let var = namespace.look_up_variable(name);
            // we set this register as equivalent to another register
            // it is not a load, because that would be superfluous
            // the expression is literally just referring to this specific register
            vec![Op::new_with_comment(
                Opcode::RMove(return_register.into(), var.into()),
                exp.span,
                "variable expression",
            )]
        }
        a => todo!("{:?}", a),
    }
}

pub(crate) fn convert_code_block_to_asm<'sc>(
    block: &TypedCodeBlock<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
    // Where to put the return value of this code block, if there was any.
    return_register: Option<&AsmRegister>,
) -> Vec<Op<'sc>> {
    let mut asm_buf: Vec<Either<JumpPlaceholer, Op<'sc>>> = vec![];
    for node in &block.contents {
        // If this is a return, then we jump to the end of the function and put the
        // value in the return register
        let res = convert_node_to_asm(node, namespace, register_sequencer, return_register);
        match res {
            NodeAsmResult::JustAsm(ops) => {
                asm_buf.append(&mut ops.into_iter().map(Either::Right).collect())
            }
            NodeAsmResult::ReturnStatement { asm } => {
                // insert a placeholder to jump to the end of the block and put the register
                // being returned in the return register
                if let Some(return_register) = return_register {
                    asm_buf.push(Op::register_move(
                        return_register.into(),
                        register_being_returned.into(),
                        node.span,
                    ));
                    // to later replace with a jump to the end of the block
                }
            }
        }
    }

    asm_buf
}

/// Initializes [Literal] `lit` into [AsmRegister] `return_register`.
fn convert_literal_to_asm<'sc>(
    lit: &Literal<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    return_register: &AsmRegister,
    _register_sequencer: &mut RegisterSequencer,
    span: Span<'sc>,
) -> Vec<Op<'sc>> {
    // first, insert the literal into the data section
    let data_id = namespace.insert_data_value(lit);
    // then get that literal id and use it to make a load word op
    vec![Op::new_with_comment(
        Opcode::Lw(return_register.into(), "$r0".to_string(), data_id),
        span,
        "literal instantiation",
    )]
}

/// For now, all functions are handled by inlining at the time of application.
fn convert_fn_app_to_asm<'sc>(
    name: &CallPath<'sc>,
    arguments: &[(Ident<'sc>, TypedExpression<'sc>)],
    function_body: &TypedCodeBlock<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    return_register: &AsmRegister,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    let mut asm_buf = vec![];
    let mut args_and_registers: HashMap<Ident<'sc>, AsmRegister> = Default::default();
    // evaluate every expression being passed into the function
    for (name, arg) in arguments {
        let return_register = register_sequencer.next();
        let ops = convert_expression_to_asm(arg, namespace, &return_register, register_sequencer);
        asm_buf.append(&mut ops);
        args_and_registers.insert(name.clone(), return_register);
    }

    // insert the arguments into the asm namespace with their registers mapped
    for (name, reg) in args_and_registers {
        namespace.insert_variable(name, reg);
    }

    // evaluate the function body
    asm_buf.append(&mut convert_code_block_to_asm(
        function_body,
        namespace,
        register_sequencer,
    ));

    todo!()
}
