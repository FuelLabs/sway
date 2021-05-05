#![allow(warnings)]

use super::*;
use crate::{
    asm_lang::*,
    error::*,
    parse_tree::{AsmExpression, AsmOp, AsmRegisterDeclaration, CallPath, UnaryOp},
    types::ResolvedType,
};
use crate::{
    parse_tree::Literal,
    semantics::{
        ast_node::{TypedAsmRegisterDeclaration, TypedCodeBlock, TypedExpressionVariant},
        TypedExpression,
    },
};
use pest::Span;
pub(crate) fn convert_subfield_expression_to_asm<'sc>(
    unary_op: &Option<UnaryOp>,
    span: &Span<'sc>,
    name: &[Ident<'sc>],
    resolved_type_of_parent: &ResolvedType<'sc>,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    return err(
        vec![],
        vec![CompileError::Unimplemented(
            "Struct field access ASM generation is unimplemented.",
            span.clone(),
        )],
    );
}
