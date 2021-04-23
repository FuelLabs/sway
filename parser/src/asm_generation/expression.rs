use super::*;
use crate::{parse_tree::CallPath, vendored_vm::*};
use crate::{
    parse_tree::Literal,
    semantics::{
        ast_node::{TypedCodeBlock, TypedExpressionVariant},
        TypedExpression,
    },
};
use pest::Span;

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
        TypedExpressionVariant::FunctionApplication { name, arguments } => convert_fn_app_to_asm(
            name,
            arguments,
            namespace,
            return_register,
            register_sequencer,
        ),
        TypedExpressionVariant::VariableExpression { unary_op, name } => {
            let var = namespace.look_up_variable(name);
            todo!()
        }
        a => todo!("{:?}", a),
    }
}

pub(crate) fn convert_code_block_to_asm<'sc>(
    block: &TypedCodeBlock<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    let mut asm_buf = vec![];
    for node in &block.contents {
        asm_buf.append(&mut convert_node_to_asm(
            node,
            namespace,
            register_sequencer,
        ));
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
    namespace: &mut AsmNamespace<'sc>,
    return_register: &AsmRegister,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    let mut asm_buf = vec![];
    let mut args_and_registers: HashMap<Ident<'sc>, AsmRegister> = Default::default();
    // evaluate every expression being passed into the function
    for (name, arg) in arguments {
        let return_register = register_sequencer.next();
        asm_buf.append(&mut convert_expression_to_asm(
            arg,
            namespace,
            &return_register,
            register_sequencer,
        ));
        args_and_registers.insert(name.clone(), return_register);
    }

    for (name, reg) in args_and_registers {
        namespace.insert_variable(name, reg);
    }

    // insert the arguments into the asm namespace with their registers mapped

    // evaluate the function body, with the

    todo!()
}
