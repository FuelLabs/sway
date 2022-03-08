#![allow(warnings)]

use super::*;
use crate::{
    asm_lang::*,
    error::*,
    parse_tree::{AsmExpression, AsmOp, AsmRegisterDeclaration, CallPath, Literal, UnaryOp},
    semantic_analysis::{
        ast_node::{
            OwnedTypedEnumVariant, OwnedTypedStructField, TypedAsmRegisterDeclaration,
            TypedCodeBlock, TypedEnumVariant, TypedExpressionVariant, TypedStructExpressionField,
            TypedStructField,
        },
        TypedExpression,
    },
    type_engine::{look_up_type_id, TypeId},
};
use expression::structs::ContiguousMemoryLayoutDescriptor;
use sway_types::{ident::Ident, span::Span};

pub(crate) fn convert_subfield_expression_to_asm(
    span: &Span,
    parent: &TypedExpression,
    field_to_access_name: &str,
    field_to_access_span: Span,
    resolved_type_of_parent: TypeId,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    return_register: &VirtualRegister,
) -> CompileResult<Vec<Op>> {
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
    let fields_for_layout = get_subfields_for_layout(resolved_type_of_parent);

    let descriptor = check!(
        get_contiguous_memory_layout(&fields_for_layout[..]),
        return err(warnings, errors),
        warnings,
        errors
    );

    asm_buf.append(&mut check!(
        convert_subfield_to_asm(
            prefix_reg,
            &field_to_access_name,
            field_to_access_span,
            return_register.clone(),
            &fields_for_layout,
            &descriptor,
        ),
        vec![],
        warnings,
        errors
    ));

    return ok(asm_buf, warnings, errors);
}

pub(crate) fn get_subfields_for_layout(
    resolved_type_of_parent: TypeId,
) -> Vec<(TypeId, Span, String)> {
    // TODO(static span): str should be ident below
    let span = sway_types::span::Span {
        span: pest::Span::new(
            "TODO(static span): use Idents instead of Strings".into(),
            0,
            0,
        )
        .unwrap(),
        path: None,
    };

    match look_up_type_id(resolved_type_of_parent) {
        TypeInfo::Struct { fields, .. } => fields
            .iter()
            .map(|OwnedTypedStructField { name, r#type, .. }| (*r#type, span.clone(), name.clone()))
            .collect::<Vec<_>>(),
        TypeInfo::Tuple(elems) => elems
            .iter()
            .enumerate()
            .map(|(pos, elem)| (*elem, span.clone(), format!("{}", pos)))
            .collect::<Vec<_>>(),
        _ => {
            unreachable!("Accessing a field or element on non-viable type should be caught during type checking.")
        }
    }
}

pub(crate) fn convert_subfield_to_asm(
    prefix_reg: VirtualRegister,
    field_to_access_name: &str,
    field_to_access_span: Span,
    return_register: VirtualRegister,
    fields_for_layout: &[(TypeId, Span, String)],
    descriptor: &ContiguousMemoryLayoutDescriptor<String>,
) -> CompileResult<Vec<Op>> {
    let mut asm_buf = vec![];
    let mut warnings = vec![];
    let mut errors = vec![];

    // step 2
    let offset_in_words = check!(
        descriptor.offset_to_field_name(field_to_access_name, field_to_access_span.clone()),
        0,
        warnings,
        errors
    );

    // TODO(static span): name_for_this_field should be span_for_this_field
    let (type_of_this_field, name_for_this_field) = fields_for_layout
        .into_iter()
        .find_map(|(ty, _span, name)| {
            if name.as_str() == field_to_access_name {
                Some((ty, name))
            } else {
                None
            }
        })
        .expect(
            "Accessing a subfield that is not no the struct would be caught during type checking",
        );

    let span = sway_types::span::Span {
        span: pest::Span::new("TODO(static span): use span_for_this_field".into(), 0, 0).unwrap(),
        path: None,
    };
    // step 3
    // if this is a copy type (primitives that fit in a word), copy it into the register.
    // Otherwise, load the pointer to the field into the register
    let resolved_type_of_this_field = match resolve_type(*type_of_this_field, &span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e.into());
            return err(warnings, errors);
        }
    };

    let size_of_type = resolved_type_of_this_field
        .size_in_words(&field_to_access_span)
        .unwrap_or_else(|e| {
            errors.push(e);
            0
        });

    asm_buf.push(if size_of_type <= 1 {
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
                look_up_type_id(*type_of_this_field).friendly_type_str()
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

    ok(asm_buf, warnings, errors)
}
