use super::*;
use crate::{
    asm_generation::{
        convert_expression_to_asm, expression::get_struct_memory_layout, AsmNamespace,
        RegisterSequencer,
    },
    semantic_analysis::ast_node::TypedReassignment,
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
            // change the lhs type to be a tuple containing both the name and the type, so we can
            // get the struct layout more easily
            // get the field layout
            // find the address of this field
            // write rhs to the address above
            // struct field reassignment
            let fields = match reassignment.lhs[0].r#type {
                MaybeResolvedType::Resolved(ResolvedType::Struct {
                    ref fields, name, ..
                }) => fields,
                a => {
                    errors.push(CompileError::NotAStruct {
                        name: reassignment.lhs[0].name.primary_name.to_string(),
                        span: reassignment.lhs[0].name.span.clone(),
                        actually: a.friendly_type_str(),
                    });
                    return err(warnings, errors);
                }
            };
            let field_layout = get_struct_memory_layout(fields);
            let lhs = reassignment.lhs.clone();
            errors.push(CompileError::Unimplemented(
                "Struct field reassignment assembly generation has not yet been implemented.",
                lhs.iter().fold(lhs[0].span(), |acc, this| {
                    crate::utils::join_spans(acc, this.span())
                }),
            ));
            return err(warnings, errors);
        }
    }

    ok(buf, warnings, errors)
}
