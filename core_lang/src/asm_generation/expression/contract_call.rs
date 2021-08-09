use super::*;
use crate::error::*;
use crate::semantic_analysis::ast_node::*;
/// Converts a function application of a contract ABI function into assembly
pub(crate) fn convert_contract_call_to_asm<'sc>(
    fn_decl: TypedFunctionDeclaration<'sc>,
    arguments: &[(Ident<'sc>, TypedExpression<'sc>)],
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    // step 0. get the function selector from the declaration
    // step 1. evaluate the arguments
    // step 2. construct the CALL op using the arguments
    todo!("above steps")
}
