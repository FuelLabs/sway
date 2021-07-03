#![allow(warnings)]

use super::*;
use crate::{
    asm_lang::*,
    error::*,
    parse_tree::{AsmExpression, AsmOp, AsmRegisterDeclaration, CallPath, UnaryOp},
    types::{MaybeResolvedType, ResolvedType},
};
use crate::{
    parse_tree::Literal,
    semantic_analysis::{
        ast_node::{
            TypedAsmRegisterDeclaration, TypedCodeBlock, TypedExpressionVariant,
            TypedStructExpressionField, TypedStructField,
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
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    // step 0. find the type and register of the prefix
    // step 1. get the memory layout of the struct
    // step 2. calculate the offset to the spot we are accessing
    // step 3. write a pointer to that word into the return register

    // step 0
    let mut asm_buf = vec![];
    let mut warnings = vec![];
    let mut errors = vec![];
    let prefix_reg = register_sequencer.next();
    let mut prefix_ops = type_check!(
        convert_expression_to_asm(parent, namespace, &prefix_reg, register_sequencer),
        vec![],
        warnings,
        errors
    );

    asm_buf.append(&mut prefix_ops);

    // now the pointer to the struct is in the prefix_reg, and we can access the subfield off
    // of that address
    // step 1
    let fields = match resolved_type_of_parent {
        MaybeResolvedType::Resolved(ResolvedType::Struct { fields, .. }) => fields,
        _ => {
            unreachable!("Accessing a field on a non-struct should be caught during type checking.")
        }
    };
    let fields_for_layout = fields
        .iter()
        .map(|TypedStructField { name, r#type, .. }| {
            (MaybeResolvedType::Resolved(r#type.clone()), name)
        })
        .collect::<Vec<_>>();
    let descriptor = type_check!(
        get_struct_memory_layout(&fields_for_layout[..]),
        return err(warnings, errors),
        warnings,
        errors
    );

    // step 2
    let offset = type_check!(
        descriptor.offset_to_field_name(&field_to_access.name),
        0,
        warnings,
        errors
    );

    // step 3
    todo!("write the LW for this field");

    return err(
        vec![],
        vec![CompileError::Unimplemented(
            "Struct field access ASM generation is unimplemented.",
            span.clone(),
        )],
    );
}
