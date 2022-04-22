use super::*;
use crate::{
    asm_generation::{
        convert_expression_to_asm, expression::get_contiguous_memory_layout, AsmNamespace,
        RegisterSequencer,
    },
    asm_lang::{VirtualImmediate12, VirtualOp},
    constants::VM_WORD_SIZE,
    semantic_analysis::ast_node::{ReassignmentLhs, TypedReassignment, TypedStructField},
    type_engine::*,
    type_engine::{resolve_type, TypeInfo},
};
use either::Either;

pub(crate) fn convert_reassignment_to_asm(
    reassignment: &TypedReassignment,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    // 0. evaluate the RHS of the reassignment
    // 1. Find the register that the previous var was stored in
    // 2. move the return register of the RHS into the register in the namespace

    let mut buf = vec![];
    let mut warnings = vec![];
    let mut errors = vec![];
    // step 0
    let return_register = register_sequencer.next();
    let mut rhs = check!(
        convert_expression_to_asm(
            &reassignment.rhs,
            namespace,
            &return_register,
            register_sequencer
        ),
        vec![],
        warnings,
        errors
    );

    buf.append(&mut rhs);

    match reassignment.lhs.len() {
        0 => unreachable!(),
        1 => {
            // step 1
            let var_register = check!(
                namespace.look_up_variable(&reassignment.lhs[0].name),
                return err(warnings, errors),
                warnings,
                errors
            );

            // step 2
            buf.push(Op::register_move_comment(
                var_register.clone(),
                return_register,
                reassignment
                    .lhs
                    .iter()
                    .fold(reassignment.lhs[0].span(), |acc, this| {
                        sway_types::Span::join(acc, this.span())
                    }),
                format!(
                    "variable {} reassignment",
                    reassignment
                        .lhs
                        .iter()
                        .map(|x| x.name.as_str())
                        .collect::<Vec<_>>()
                        .join(".")
                ),
            ));
        }
        _ => {
            // 0. get the field layout
            // 1. find the offset to this field
            // 2. write rhs to the address above
            //
            // step 0
            let mut offset_in_words = 0;
            let mut iter = reassignment.lhs.iter();
            let (mut fields, top_level_decl) = match iter
                .next()
                .map(|ReassignmentLhs { r#type, name }| -> Result<_, _> {
                    match resolve_type(*r#type, name.span()) {
                        Ok(TypeInfo::Struct { ref fields, .. }) => Ok((fields.clone(), name)),
                        Ok(ref a) => Err(CompileError::NotAStruct {
                            name: name.as_str().to_string(),
                            span: name.span().clone(),
                            actually: a.friendly_type_str(),
                        }),
                        Err(a) => Err(CompileError::TypeError(a)),
                    }
                })
                .expect("Empty structs not allowed yet")
            {
                Ok(o) => o,
                Err(e) => {
                    errors.push(e);
                    return err(warnings, errors);
                }
            };

            // delve into this potentially nested field access and figure out the location of this
            // subfield
            for ReassignmentLhs { r#type, name } in iter {
                let r#type = match resolve_type(*r#type, name.span()) {
                    Ok(o) => o,
                    Err(e) => {
                        errors.push(CompileError::TypeError(e));
                        TypeInfo::ErrorRecovery
                    }
                };
                // TODO(static span) use spans instead of strings below
                let span = sway_types::span::Span::new(
                    "TODO(static span): use Idents instead of Strings".into(),
                    0,
                    0,
                    None,
                )
                .unwrap();
                let fields_for_layout = fields
                    .iter()
                    .map(|TypedStructField { name, r#type, .. }| {
                        (*r#type, span.clone(), name.clone())
                    })
                    .collect::<Vec<_>>();
                let field_layout = check!(
                    get_contiguous_memory_layout(&fields_for_layout[..]),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let offset_of_this_field = check!(
                    field_layout.offset_to_field_name(name.as_str(), name.span().clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                offset_in_words += offset_of_this_field;
                fields = match r#type {
                    TypeInfo::Struct { ref fields, .. } => fields.clone(),
                    a => {
                        errors.push(CompileError::NotAStruct {
                            name: name.as_str().to_string(),
                            span: name.span().clone(),
                            actually: a.friendly_type_str(),
                        });
                        return err(warnings, errors);
                    }
                };
            }
            let ptr = check!(
                namespace.look_up_variable(top_level_decl),
                return err(warnings, errors),
                warnings,
                errors
            );

            let offset_in_bytes = offset_in_words * VM_WORD_SIZE;
            let offset_in_words =
                match VirtualImmediate12::new(offset_in_words, reassignment.rhs.span.clone()) {
                    Ok(o) => o,
                    Err(e) => {
                        errors.push(e);
                        return err(warnings, errors);
                    }
                };

            // the address to write to is:
            // the register `ptr` (the struct pointer)
            // + the offset in words (imm is in words, the vm multiplies it by 8)

            // if the size of this type is > 1 word, then we use MCP to copy the entire value to
            // the mem address pointed to by the struct.
            let size_of_ty = look_up_type_id(reassignment.rhs.return_type)
                .size_in_words(&reassignment.rhs.span)
                .unwrap_or_else(|e| {
                    errors.push(e);
                    0
                });
            match size_of_ty {
                0 => (),
                1 => {
                    buf.push(Op::write_register_to_memory(
                        ptr.clone(),
                        return_register,
                        offset_in_words,
                        sway_types::Span::join(
                            reassignment.lhs[0].span(),
                            reassignment.rhs.span.clone(),
                        ),
                    ));
                }
                size => {
                    // 0. grab the position of the data in the struct (ptr_start + offset_in_bytes)
                    // 1. MCPI current ptr, ret register, size_of_ty

                    // 0.
                    let addr_of_field = register_sequencer.next();
                    buf.push(Op {
                        opcode: Either::Left(VirtualOp::ADDI(
                            addr_of_field.clone(),
                            ptr.clone(),
                            VirtualImmediate12::new_unchecked(
                                offset_in_bytes,
                                "structs can't be this big",
                            ),
                        )),
                        comment: "reassign multiword struct field".into(),
                        owning_span: None,
                    });

                    // 1.
                    buf.push(Op {
                        opcode: Either::Left(VirtualOp::MCPI(
                            addr_of_field,
                            return_register,
                            VirtualImmediate12::new_unchecked(
                                size * VM_WORD_SIZE,
                                "structs fields can't be this big",
                            ),
                        )),
                        comment: Default::default(),
                        owning_span: None,
                    });
                }
            }
        }
    }

    ok(buf, warnings, errors)
}
