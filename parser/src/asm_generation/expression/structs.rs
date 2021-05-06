//! This module contains the logic for struct layout in memory and instantiation.

use crate::{
    asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer},
    asm_lang::{ConstantRegister, Op, Opcode, RegisterId},
    error::*,
    parse_tree::Literal,
    semantics::ast_node::TypedStructExpressionField,
    CompileResult, Ident,
};

pub(crate) fn convert_struct_expression_to_asm<'sc>(
    struct_name: &Ident<'sc>,
    fields: &[TypedStructExpressionField<'sc>],
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<'sc, Vec<Op<'sc>>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut asm_buf = vec![];
    // step 0: calculate the total size needed for the whole struct
    // step 1: store the value currently in $sp, it will become the pointer to the first field
    // step 2: use CFE to extend the call frame by the size calculated in step 0
    // step 3: for every field in the struct:
    //             evaluate its initializer
    //             SW (store word) at the current pointer
    //             increment pointer by the size of this field
    //
    // for now i dont think this step is needed, we can resolve the types at call time
    // but im leaving this here for historical purposes in case i need to come back and implement
    // step 4
    //
    // step 4: put the pointer to the beginning of the struct in the namespace

    // step 0
    let fields_with_sizes: Vec<(&TypedStructExpressionField, u64)> = fields
        .into_iter()
        .map(|field| (field, field.value.return_type.stack_size_of()))
        .collect::<Vec<_>>();
    let total_size = fields_with_sizes.iter().fold(0, |acc, (_, num)| acc + num);

    asm_buf.push(Op::new_comment(format!(
        "{} struct initialization",
        struct_name.primary_name
    )));

    // step 1
    let struct_beginning_pointer = register_sequencer.next();
    asm_buf.push(Op::unowned_register_move(
        struct_beginning_pointer.clone(),
        RegisterId::Constant(ConstantRegister::StackPointer),
    ));

    // step 2
    // decide how many stack allocations we will need to do based on the size of the struct
    // this is 2^24, the size of an immediate.
    let twenty_four_bits = 0b111111111111111111111111;
    let number_of_allocations_necessary = (total_size / twenty_four_bits) + 1;

    // construct the allocation ops
    for allocation_index in 0..number_of_allocations_necessary {
        let left_to_allocate = total_size - (allocation_index * twenty_four_bits);
        let this_allocation = if left_to_allocate > twenty_four_bits {
            twenty_four_bits
        } else {
            left_to_allocate
        };
        // since the size of `this_allocation` is bound by the size of 2^24, we know that
        // downcasting to a u32 is safe.
        asm_buf.push(Op::unowned_stack_allocate_memory(this_allocation as u32));
    }

    // step 3
    let mut offset = 0;
    // TODO:
    for TypedStructExpressionField { name, value } in fields {
        // evaluate the expression
        let return_register = register_sequencer.next();
        let mut field_instantiation = type_check!(
            convert_expression_to_asm(value, namespace, &return_register, register_sequencer),
            vec![],
            warnings,
            errors
        );
        asm_buf.append(&mut field_instantiation);
        asm_buf.push(Op::write_register_to_memory(
            struct_beginning_pointer.clone(),
            return_register,
            offset,
            name.span.clone(),
        ));
        // TODO: if the struct needs multiple allocations, this offset could exceed the size of the
        // immediate value allowed in SW. In that case, we need to shift `struct_beginning_pointer`
        // to the max offset and start the offset back from 0. This is only for structs in excess
        // of 130MB.
        offset += value.return_type.stack_size_of() as u32;
    }

    ok(asm_buf, warnings, errors)
}
