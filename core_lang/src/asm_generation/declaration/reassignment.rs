use super::*;
use crate::{
    asm_generation::{
        convert_expression_to_asm, expression::get_struct_memory_layout, AsmNamespace,
        RegisterSequencer,
    },
    semantic_analysis::ast_node::{ReassignmentLhs, TypedReassignment, TypedStructField},
    types::{MaybeResolvedType, ResolvedType},
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
    let mut rhs = type_check!(
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
            let var_register = type_check!(
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
            let mut fields = match iter
                .next()
                .map(
                    |ReassignmentLhs { r#type, name }| -> Result<_, CompileError<'sc>> {
                        match r#type {
                            MaybeResolvedType::Resolved(ResolvedType::Struct {
                                ref fields,
                                ref name,
                                ..
                            }) => Ok(fields.clone()),
                            ref a => Err(CompileError::NotAStruct {
                                name: name.primary_name.to_string(),
                                span: name.span.clone(),
                                actually: a.friendly_type_str(),
                            }),
                        }
                    },
                )
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
                let fields_for_layout = fields
                    .iter()
                    .map(|TypedStructField { name, r#type, .. }| {
                        (MaybeResolvedType::Resolved(r#type.clone()), name)
                    })
                    .collect::<Vec<_>>();
                let field_layout = type_check!(
                    get_struct_memory_layout(&fields_for_layout[..]),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let offset_of_this_field = type_check!(
                    field_layout.offset_to_field_name(name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                offset_in_words += offset_of_this_field;
                fields = match r#type {
                    MaybeResolvedType::Resolved(ResolvedType::Struct {
                        ref fields,
                        ref name,
                        ..
                    }) => fields.clone(),
                    ref a => {
                        errors.push(CompileError::NotAStruct {
                            name: name.primary_name.to_string(),
                            span: name.span.clone(),
                            actually: a.friendly_type_str(),
                        });
                        return err(warnings, errors);
                    }
                };
            }

            let ptr = todo!("find the pointer to the top-level struct itself");

            todo!("write the evaluated RHS to the LHS calculated by the ptr + the offset")
        }
    }

    ok(buf, warnings, errors)
}
