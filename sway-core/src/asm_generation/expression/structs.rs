//! This module contains the logic for struct layout in memory and instantiation.
use crate::{
    asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer},
    asm_lang::{
        ConstantRegister, Op, VirtualImmediate12, VirtualImmediate24, VirtualOp, VirtualRegister,
    },
    error::*,
    semantic_analysis::ast_node::{TypedExpression, TypedStructExpressionField},
    type_engine::{look_up_type_id, resolve_type, TypeId},
    CompileResult, Ident,
};
use sway_types::span::Span;

/// Contains an ordered array of fields and their sizes in words. Used in the code generation
/// of struct/tuple field reassignments, accesses, and struct/tuple initializations.
#[derive(Debug)]
pub(crate) struct ContiguousMemoryLayoutDescriptor<N> {
    fields: Vec<FieldMemoryLayoutDescriptor<N>>,
}

/// Describes the size, name, and type of an individual struct/tuple field in a memory layout.
#[derive(Debug)]
pub(crate) struct FieldMemoryLayoutDescriptor<N> {
    name_of_field: N,
    size: u64,
}

impl ContiguousMemoryLayoutDescriptor<Ident> {
    /// Calculates the offset in words from the start of a struct to a specific field.
    pub(crate) fn offset_to_field_name(&self, name: &str, span: Span) -> CompileResult<u64> {
        let field_ix = if let Some(ix) =
            self.fields
                .iter()
                .position(|FieldMemoryLayoutDescriptor { name_of_field, .. }| {
                    name_of_field.as_str() == name
                }) {
            ix
        } else {
            return err(vec![],
                vec![
                CompileError::Internal(
                    "Attempted to calculate struct memory offset on field that did not exist in struct.",
                    span
                    )
                ]);
        };

        ok(
            self.fields
                .iter()
                .take(field_ix)
                .fold(0, |acc, FieldMemoryLayoutDescriptor { size, .. }| {
                    acc + *size
                }),
            vec![],
            vec![],
        )
    }
}

impl<N> ContiguousMemoryLayoutDescriptor<N> {
    pub(crate) fn total_size(&self) -> u64 {
        self.fields
            .iter()
            .map(|FieldMemoryLayoutDescriptor { size, .. }| size)
            .sum()
    }
}

#[test]
fn test_struct_memory_layout() {
    let first_field_name = Ident::new_no_span("foo");
    let second_field_name = Ident::new_no_span("bar");

    let numbers = ContiguousMemoryLayoutDescriptor {
        fields: vec![
            FieldMemoryLayoutDescriptor {
                name_of_field: first_field_name.clone(),
                size: 1,
            },
            FieldMemoryLayoutDescriptor {
                name_of_field: second_field_name.clone(),
                size: 1,
            },
        ],
    };

    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    assert_eq!(numbers.total_size(), 2u64);
    assert_eq!(
        numbers
            .offset_to_field_name(first_field_name.as_str(), first_field_name.span().clone())
            .unwrap(&mut warnings, &mut errors),
        0u64
    );
    assert_eq!(
        numbers
            .offset_to_field_name(second_field_name.as_str(), first_field_name.span().clone())
            .unwrap(&mut warnings, &mut errors),
        1u64
    );
}

pub(crate) fn get_contiguous_memory_layout<N: Clone>(
    fields_with_names: &[(TypeId, Span, N)],
) -> CompileResult<ContiguousMemoryLayoutDescriptor<N>> {
    let mut fields_with_sizes = Vec::with_capacity(fields_with_names.len());
    let warnings = vec![];
    let mut errors = vec![];
    for (field, span, name) in fields_with_names {
        let ty = look_up_type_id(*field);
        let stack_size = match ty.size_in_words(span) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return err(warnings, errors);
            }
        };

        fields_with_sizes.push(FieldMemoryLayoutDescriptor {
            name_of_field: name.clone(),
            size: stack_size,
        });
    }
    ok(
        ContiguousMemoryLayoutDescriptor {
            fields: fields_with_sizes,
        },
        warnings,
        errors,
    )
}

pub(crate) fn convert_fields_to_asm<N: Clone + std::fmt::Display>(
    fields: &[(TypedExpression, Span, N)],
    struct_beginning_pointer: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
    mut asm_buf: Vec<Op>,
) -> CompileResult<Vec<Op>> {
    let mut warnings = vec![];
    let mut errors = vec![];
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

    let fields_for_layout = fields
        .iter()
        .map(|(value, span, name)| (value.return_type, span.clone(), name.clone()))
        .collect::<Vec<_>>();

    // step 0
    let descriptor = check!(
        get_contiguous_memory_layout(&fields_for_layout),
        return err(warnings, errors),
        warnings,
        errors
    );

    let total_size = descriptor.total_size();

    if total_size == 0 {
        asm_buf.push(Op::new_comment("fields have total size of zero."));
        return ok(asm_buf, warnings, errors);
    }

    if total_size == 0 {
        asm_buf.push(Op::new_comment("fields have total size of zero."));
        return ok(asm_buf, warnings, errors);
    }

    // step 1
    asm_buf.push(Op::unowned_register_move(
        struct_beginning_pointer.clone(),
        VirtualRegister::Constant(ConstantRegister::StackPointer),
    ));

    // step 2
    // decide how many call frame extensions are needed based on the size of the struct
    // and how many bits can be put in a single cfei op
    // limit struct size to 12 bits for now, for simplicity
    let twelve_bits = super::compiler_constants::TWELVE_BITS;
    let number_of_allocations_necessary = (total_size + (twelve_bits - 1)) / twelve_bits;

    // construct the allocation ops
    for allocation_index in 0..number_of_allocations_necessary {
        let left_to_allocate = total_size - (allocation_index * twelve_bits);
        let this_allocation = if left_to_allocate > twelve_bits {
            twelve_bits
        } else {
            left_to_allocate
        };
        // we call `new_unchecked` here because we have validated the size is okay above
        asm_buf.push(Op::unowned_stack_allocate_memory(
            VirtualImmediate24::new_unchecked(
                this_allocation * 8, // this_allocation is words but this op takes bytes
                "struct size was checked manually to be within 12 bits",
            ),
        ));
    }

    // step 3
    // `offset` is in words
    let mut offset = 0;
    for (value, span, name) in fields {
        // evaluate the expression
        let return_register = register_sequencer.next();
        let value_stack_size: u64 = match resolve_type(value.return_type, span) {
            Ok(o) => match o.size_in_words(span) {
                Ok(o) => o,
                Err(e) => {
                    errors.push(e);
                    return err(warnings, errors);
                }
            },
            Err(e) => {
                errors.push(e.into());
                return err(warnings, errors);
            }
        };
        let mut field_instantiation = check!(
            convert_expression_to_asm(value, namespace, &return_register, register_sequencer),
            vec![],
            warnings,
            errors
        );
        asm_buf.append(&mut field_instantiation);
        // if the value is less than one word in size, we write it via the SW opcode.
        // Otherwise, use MCPI to copy the contiguous memory
        if value_stack_size > 1 {
            // copy the struct beginning pointer and add the offset to it
            let address_to_write_to = register_sequencer.next();
            // load the address via ADDI
            asm_buf.push(Op {
                opcode: either::Either::Left(VirtualOp::ADDI(
                    address_to_write_to.clone(),
                    struct_beginning_pointer.clone(),
                    VirtualImmediate12::new_unchecked(offset * 8, "struct size is too large"),
                )),
                owning_span: Some(value.span.clone()),
                comment: format!(
                    "prep struct field reg (size {} for field {})",
                    value_stack_size, name,
                ),
            });

            // copy the data
            asm_buf.push(Op {
                opcode: either::Either::Left(VirtualOp::MCPI(
                    address_to_write_to,
                    return_register,
                    VirtualImmediate12::new_unchecked(
                        value_stack_size * 8,
                        "struct cannot be this big",
                    ),
                )),
                owning_span: Some(value.span.clone()),
                comment: format!("cp type size {} for field {}", value_stack_size, name),
            });
        } else {
            asm_buf.push(Op::write_register_to_memory(
                struct_beginning_pointer.clone(),
                return_register,
                VirtualImmediate12::new_unchecked(offset, "the whole struct is less than 12 bits so every individual field should be as well."),
                span.clone(),
            ));
        }
        // TODO: if the struct needs multiple allocations, this offset could exceed the size of the
        // immediate value allowed in SW. In that case, we need to shift `struct_beginning_pointer`
        // to the max offset and start the offset back from 0. This is only for structs in excess
        // of 130MB
        // from john about the above: As a TODO, maybe let's just restrict the maximum size of
        // something (I don't know exactly what) at the consensus level so this case is guaranteed
        // to never be hit.
        offset += value_stack_size;
    }

    ok(asm_buf, warnings, errors)
}

pub(crate) fn convert_struct_expression_to_asm(
    struct_name: &Ident,
    fields: &[TypedStructExpressionField],
    struct_beginning_pointer: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let fields = fields
        .iter()
        .map(|TypedStructExpressionField { name, value }| {
            (
                value.clone(),
                name.span().clone(),
                name.as_str().to_string(),
            )
        })
        .collect::<Vec<_>>();

    let asm_buf = vec![Op::new_comment(format!(
        "{} struct initialization",
        struct_name.as_str()
    ))];

    convert_fields_to_asm(
        &fields,
        struct_beginning_pointer,
        namespace,
        register_sequencer,
        asm_buf,
    )
}

pub(crate) fn convert_tuple_expression_to_asm(
    fields: &[TypedExpression],
    tuple_beginning_pointer: &VirtualRegister,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    let fields = fields
        .iter()
        .enumerate()
        .map(|(i, field)| (field.clone(), field.span.clone(), i))
        .collect::<Vec<_>>();
    let asm_buf = vec![Op::new_comment(format!(
        "{}-tuple initialization",
        fields.len(),
    ))];

    convert_fields_to_asm(
        &fields,
        tuple_beginning_pointer,
        namespace,
        register_sequencer,
        asm_buf,
    )
}
