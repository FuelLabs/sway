#![allow(warnings)]

use super::*;
use crate::{
    asm_lang::*,
    error::*,
    parse_tree::{AsmExpression, AsmOp, AsmRegisterDeclaration, CallPath, UnaryOp},
    types::MaybeResolvedType,
};
use crate::{
    parse_tree::Literal,
    semantic_analysis::{
        ast_node::{
            TypedAsmRegisterDeclaration, TypedCodeBlock, TypedExpressionVariant, TypedStructField,
        },
        TypedExpression,
    },
};
use pest::Span;
pub(crate) fn convert_subfield_expression_to_asm<'sc>(
    unary_op: &Option<UnaryOp>,
    span: &Span<'sc>,
    parent: &TypedExpression<'sc>,
    field_to_access: &TypedStructField<'sc>,
    resolved_type_of_parent: &MaybeResolvedType<'sc>,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    // step 0. find the type of the top level
    return err(
        vec![],
        vec![CompileError::Unimplemented(
            "Struct field access ASM generation is unimplemented.",
            span.clone(),
        )],
    );
}
