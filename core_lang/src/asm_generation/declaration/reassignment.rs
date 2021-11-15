use super::*;
use crate::{
    asm_generation::{
        convert_expression_to_asm, expression::get_struct_memory_layout, AsmNamespace,
        RegisterSequencer,
    },
    asm_lang::VirtualImmediate12,
    semantic_analysis::ast_node::{OwnedTypedStructField, ReassignmentLhs, TypedReassignment},
    type_engine::{resolve_type, TypeInfo},
};

pub(crate) fn convert_reassignment_to_asm<'sc>(
    reassignment: &TypedReassignment<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
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
                        crate::utils::join_spans(acc, this.span())
                    }),
                format!(
                    "variable {} reassignment",
                    reassignment
                        .lhs
                        .iter()
                        .map(|x| x.name.primary_name)
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
                    match resolve_type(*r#type, &name.span) {
                        Ok(TypeInfo::Struct { ref fields, .. }) => Ok((fields.clone(), name)),
                        Ok(ref a) => Err(CompileError::NotAStruct {
                            name: name.primary_name.to_string(),
                            span: name.span.clone(),
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
                let r#type = match resolve_type(*r#type, &name.span) {
                    Ok(o) => o,
                    Err(e) => {
                        errors.push(CompileError::TypeError(e));
                        TypeInfo::ErrorRecovery
                    }
                };
                // TODO(static span) use spans instead of strings below
                let fields_for_layout = fields
                    .iter()
                    .map(|OwnedTypedStructField { name, r#type, .. }| (*r#type, name.as_str()))
                    .collect::<Vec<_>>();
                let field_layout = check!(
                    get_struct_memory_layout(&fields_for_layout[..]),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let offset_of_this_field = check!(
                    field_layout.offset_to_field_name(name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                offset_in_words += offset_of_this_field;
                fields = match r#type {
                    TypeInfo::Struct { ref fields, .. } => fields.clone(),
                    a => {
                        errors.push(CompileError::NotAStruct {
                            name: name.primary_name.to_string(),
                            span: name.span.clone(),
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

            buf.push(Op::write_register_to_memory(
                ptr.clone(),
                return_register,
                offset_in_words,
                crate::utils::join_spans(reassignment.lhs[0].span(), reassignment.rhs.span.clone()),
            ));
        }
    }

    ok(buf, warnings, errors)
}
