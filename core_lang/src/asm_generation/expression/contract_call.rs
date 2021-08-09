use super::*;
use crate::error::*;
use crate::semantic_analysis::ast_node::*;
/// Converts a function application of a contract ABI function into assembly
pub(crate) fn convert_contract_call_to_asm<'sc>(
    selector: [u8; 4],
    cgas: &TypedExpression<'sc>,
    bal: &TypedExpression<'sc>,
    coin_color: &TypedExpression<'sc>,
    user_argument: &TypedExpression<'sc>,
    register_sequencer: &mut RegisterSequencer,
    namespace: &mut AsmNamespace<'sc>,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    // step 0. evaluate the arguments
    // step 1. construct the CALL op using the arguments
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![];
    // step 0
    //
    let user_argument_register = register_sequencer.next();
    let cgas_register = register_sequencer.next();
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
        convert_expression_to_asm(cgas, namespace, &cgas_register, register_sequencer),
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
    todo!("above steps")
}
