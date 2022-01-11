use either::Either;

use crate::asm_generation::{
    compiler_constants::*, convert_expression_to_asm, AsmNamespace, RegisterSequencer,
};
use crate::asm_lang::{
    ConstantRegister, Op, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24, VirtualOp,
    VirtualRegister,
};
use crate::type_engine::{look_up_type_id, TypeId};
use crate::Span;
use crate::{
    error::*,
    semantic_analysis::{ast_node::TypedEnumDeclaration, TypedExpression},
    type_engine::resolve_type,
    CompileResult, Ident, Literal,
};

pub(crate) fn convert_enum_instantiation_to_asm(
    decl: &TypedEnumDeclaration,
    variant_name: &Ident,
    tag: usize,
    contents: &Option<Box<TypedExpression>>,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    // step 0: load the tag into a register
    // step 1: load the data into a register
    // step 2: write both registers sequentially to memory, extending the call frame
    // step 3: write the location of the value to the return register
    let mut asm_buf = vec![];
    // step 0
    let data_label = namespace.insert_data_value(&Literal::U64(tag as u64));
    let tag_register = register_sequencer.next();
    asm_buf.push(Op::unowned_load_data_comment(
        tag_register.clone(),
        data_label,
        format!("{} enum instantiation", decl.name.as_str()),
    ));
    let pointer_register = register_sequencer.next();
    // copy stack pointer into pointer register
    asm_buf.push(Op::unowned_register_move_comment(
        pointer_register.clone(),
        VirtualRegister::Constant(ConstantRegister::StackPointer),
        "load $sp for enum pointer",
    ));
    let ty = match resolve_type(decl.as_type(), &decl.span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e.into());
            return err(warnings, errors);
        }
    };
    let size_of_enum: u64 = 1 /* tag */ + match ty.size_in_words(variant_name.span()) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };
    if size_of_enum > EIGHTEEN_BITS {
        errors.push(CompileError::Unimplemented(
            "Stack variables which exceed 2^18 words in size are not supported yet.",
            decl.clone().span,
        ));
        return err(warnings, errors);
    }

    asm_buf.push(Op::unowned_stack_allocate_memory(
        VirtualImmediate24::new_unchecked(
            size_of_enum * 8,
            "this size is manually checked to be lower than 2^24",
        ),
    ));
    // initialize all the memory to 0
    // there are only 18 bits of immediate in MCLI so we need to do this in multiple passes,
    // This is not yet implemented, so instead we just limit enum size to 2^18 words
    asm_buf.push(Op::new(
        VirtualOp::MCLI(
            pointer_register.clone(),
            VirtualImmediate18::new_unchecked(
                size_of_enum,
                "the enum was manually checked to be under 2^18 words in size",
            ),
        ),
        decl.clone().span,
    ));
    // write the tag
    // step 2
    asm_buf.push(Op::write_register_to_memory(
        pointer_register.clone(),
        tag_register,
        VirtualImmediate12::new_unchecked(0, "constant num; infallible"),
        decl.clone().span,
    ));

    // step 1 continued
    // // if there are any enum contents, instantiate them
    if let Some(instantiation) = contents {
        let return_register = register_sequencer.next();
        let mut asm = check!(
            convert_expression_to_asm(
                &*instantiation,
                namespace,
                &return_register,
                register_sequencer
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        asm_buf.append(&mut asm);
        // write these enum contents to the address after the tag
        // step 2
        asm_buf.push(Op::write_register_to_memory_comment(
            pointer_register.clone(),
            return_register,
            VirtualImmediate12::new_unchecked(1, "this is the constant 1; infallible"), // offset by 1 because the tag was already written
            instantiation.span.clone(),
            format!("{} enum contents", decl.name.as_str()),
        ));
    }

    // step 3
    asm_buf.push(Op::register_move(
        return_register.clone(),
        pointer_register,
        decl.clone().span,
    ));

    ok(asm_buf, warnings, errors)
}

pub(crate) fn convert_enum_arg_expression_to_asm(
    span: &Span,
    parent: &TypedExpression,
    arg_type: TypeId,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    return_register: &VirtualRegister,
) -> CompileResult<Vec<Op>> {
    // step 0. find the type and register of the prefix
    // step 1. calculate the offset to the spot we are accessing
    // step 2. write a pointer to that word into the return register

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

    // steps 1 + 2 :)
    // if this is a copy type (primitives that fit in a word), copy it into the register.
    // Otherwise, load the pointer to the field into the register
    let resolved_type_of_this_arg = match resolve_type(arg_type, &span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e.into());
            return err(warnings, errors);
        }
    };
    let the_op = if resolved_type_of_this_arg.is_copy_type() {
        let offset_in_words = match VirtualImmediate12::new(1, span.clone()) {
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
                look_up_type_id(arg_type).friendly_type_str()
            ),
            owning_span: Some(span.clone()),
        }
    } else {
        let offset_in_bytes = match VirtualImmediate12::new(8, span.clone()) {
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
            comment: "Construct pointer for enum arg".into(),
            owning_span: Some(span.clone()),
        }
    };
    asm_buf.push(the_op);

    ok(asm_buf, warnings, errors)
}
