#![allow(warnings)]

use super::*;
use crate::span::Span;
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

pub(crate) fn convert_subfield_expression_to_asm<'sc>(
    unary_op: &Option<UnaryOp>,
    span: &Span<'sc>,
    parent: &TypedExpression<'sc>,
    field_to_access: &TypedStructField<'sc>,
    resolved_type_of_parent: &MaybeResolvedType<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
    return_register: &VirtualRegister,
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
    let mut prefix_ops = check!(
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
    let descriptor = check!(
        get_struct_memory_layout(&fields_for_layout[..]),
        return err(warnings, errors),
        warnings,
        errors
    );

    // step 2
    let offset_in_words = check!(
        descriptor.offset_to_field_name(&field_to_access.name),
        0,
        warnings,
        errors
    );

    let type_of_this_field = fields_for_layout
        .into_iter()
        .find_map(|(ty, name)| {
            if *name == field_to_access.name {
                Some(ty)
            } else {
                None
            }
        })
        .expect(
            "Accessing a subfield that is not no the struct would be caught during type checking",
        );

    // step 3
    // if this is a copy type (primitives that fit in a word), copy it into the register.
    // Otherwise, load the pointer to the field into the register
    asm_buf.push(if type_of_this_field.is_copy_type() {
        let offset_in_words = match VirtualImmediate12::new(offset_in_words, span.clone()) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return err(warnings, errors);
            }
        };

        Op {
            opcode: Either::Left(VirtualOp::LW(
                return_register.clone(),
                prefix_reg,
                offset_in_words,
            )),
            comment: format!(
                "Loading copy type: {}",
                type_of_this_field.friendly_type_str()
            ),
            owning_span: Some(span.clone()),
        }
    } else {
        // Load the offset, plus the actual memory address of the struct, as a pointer
        // into the register
        //
        // first, construct the pointer by adding the offset to the pointer from the prefix
        let offset_in_bytes = match VirtualImmediate12::new(offset_in_words * 8, span.clone()) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return err(warnings, errors);
            }
        };

        Op {
            opcode: Either::Left(VirtualOp::ADDI(
                return_register.clone(),
                prefix_reg,
                offset_in_bytes,
            )),
            comment: "Construct pointer for struct field".into(),
            owning_span: Some(span.clone()),
        }
    });

    return ok(asm_buf, warnings, errors);
}
