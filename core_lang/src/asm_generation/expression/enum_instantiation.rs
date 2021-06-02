use crate::asm_generation::{
    compiler_constants::*, convert_expression_to_asm, AsmNamespace, RegisterSequencer,
};
use crate::asm_lang::{
    virtual_ops::{
        ConstantRegister, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24, VirtualOp,
        VirtualRegister,
    },
    Op,
};
use crate::error::*;
use crate::semantic_analysis::ast_node::TypedEnumDeclaration;
use crate::semantic_analysis::TypedExpression;
use crate::Literal;
use crate::{CompileResult, Ident};

pub(crate) fn convert_enum_instantiation_to_asm<'sc>(
    decl: &TypedEnumDeclaration<'sc>,
    _variant_name: &Ident<'sc>,
    tag: usize,
    contents: &Option<Box<TypedExpression<'sc>>>,
    return_register: &VirtualRegister,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
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
        format!("{} enum instantiation", decl.name.primary_name),
    ));
    let pointer_register = register_sequencer.next();
    // copy stack pointer into pointer register
    asm_buf.push(Op::unowned_register_move_comment(
        pointer_register.clone(),
        VirtualRegister::Constant(ConstantRegister::StackPointer),
        "load $sp for enum pointer",
    ));
    let size_of_enum = 1 /* tag */ + decl.as_type().stack_size_of();
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
        tag_register.clone(),
        VirtualImmediate12::new_unchecked(0, "constant num; infallible"),
        decl.clone().span,
    ));

    // step 1 continued
    // // if there are any enum contents, instantiate them
    if let Some(instantiation) = contents {
        let return_register = register_sequencer.next();
        let mut asm = type_check!(
            convert_expression_to_asm(
                &*instantiation,
                namespace,
                &return_register.clone(),
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
            return_register.clone(),
            VirtualImmediate12::new_unchecked(1, "this is the constant 1; infallible"), // offset by 1 because the tag was already written
            instantiation.span.clone(),
            format!("{} enum contents", decl.name.primary_name),
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
