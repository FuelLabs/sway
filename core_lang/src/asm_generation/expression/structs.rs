//! This module contains the logic for struct layout in memory and instantiation.
use crate::{
    asm_generation::{convert_expression_to_asm, AsmNamespace, RegisterSequencer},
    asm_lang::{
        ConstantRegister, Op, VirtualImmediate12, VirtualImmediate24, VirtualOp, VirtualRegister,
    },
    error::*,
    semantic_analysis::ast_node::TypedStructExpressionField,
    type_engine::{look_up_type_id, resolve_type, TypeId},
    CompileResult, Ident,
};

/// Contains an ordered array of fields and their sizes in words. Used in the code generation
/// of struct field reassignments, accesses, and struct initializations.
#[derive(Debug)]
pub(crate) struct StructMemoryLayoutDescriptor {
    fields: Vec<StructFieldMemoryLayoutDescriptor>,
}

/// Describes the size, name, and type of an individual struct field in a memory layout.
#[derive(Debug)]
pub(crate) struct StructFieldMemoryLayoutDescriptor {
    // TODO(static span) this should be an ident
    name_of_field: String,
    size: u64,
}

impl StructMemoryLayoutDescriptor {
    /// Calculates the offset in words from the start of a struct to a specific field.
    pub(crate) fn offset_to_field_name<'sc>(&self, name: &Ident<'sc>) -> CompileResult<'sc, u64> {
        let field_ix = if let Some(ix) = self.fields.iter().position(
            |StructFieldMemoryLayoutDescriptor { name_of_field, .. }| {
                name_of_field.as_str() == name.primary_name
            },
        ) {
            ix
        } else {
            return err(vec![],
                vec![
                CompileError::Internal(
                    "Attempted to calculate struct memory offset on field that did not exist in struct.",
                    name.span.clone()
                    )
                ]);
        };

        ok(
            self.fields
                .iter()
                .take(field_ix)
                .fold(0, |acc, StructFieldMemoryLayoutDescriptor { size, .. }| {
                    acc + *size
                }),
            vec![],
            vec![],
        )
    }
    pub(crate) fn total_size(&self) -> u64 {
        self.fields
            .iter()
            .map(|StructFieldMemoryLayoutDescriptor { size, .. }| size)
            .sum()
    }
}

#[test]
fn test_struct_memory_layout() {
    use crate::span::Span;
    let first_field_name = Ident {
        span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        primary_name: "foo",
    };
    let second_field_name = Ident {
        span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        primary_name: "bar",
    };

    let numbers = StructMemoryLayoutDescriptor {
        fields: vec![
            StructFieldMemoryLayoutDescriptor {
                name_of_field: first_field_name.primary_name.to_string(),
                size: 1,
            },
            StructFieldMemoryLayoutDescriptor {
                name_of_field: second_field_name.primary_name.to_string(),
                size: 1,
            },
        ],
    };

    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    assert_eq!(numbers.total_size(), 2u64);
    assert_eq!(
        numbers
            .offset_to_field_name(&first_field_name)
            .unwrap(&mut warnings, &mut errors),
        0u64
    );
    assert_eq!(
        numbers
            .offset_to_field_name(&second_field_name)
            .unwrap(&mut warnings, &mut errors),
        1u64
    );
}

pub(crate) fn get_struct_memory_layout<'sc>(
    fields_with_names: &[(TypeId, &str)],
) -> CompileResult<'sc, StructMemoryLayoutDescriptor> {
    let span = crate::Span {
        span: pest::Span::new("TODO(static span): use Idents instead of Strings", 0, 0).unwrap(),
        path: None,
    };
    let mut fields_with_sizes = vec![];
    let warnings = vec![];
    let mut errors = vec![];
    for (field, name) in fields_with_names {
        let ty = look_up_type_id(*field);
        let stack_size = match ty.stack_size_of(&span) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e);
                return err(warnings, errors);
            }
        };

        fields_with_sizes.push(StructFieldMemoryLayoutDescriptor {
            name_of_field: name.to_string(),
            size: stack_size,
        });
    }
    ok(
        StructMemoryLayoutDescriptor {
            fields: fields_with_sizes,
        },
        warnings,
        errors,
    )
}

pub(crate) fn convert_struct_expression_to_asm<'sc>(
    struct_name: &Ident<'sc>,
    fields: &[TypedStructExpressionField<'sc>],
    struct_beginning_pointer: &VirtualRegister,
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
    let fields_for_layout = fields
        .iter()
        .map(|TypedStructExpressionField { name, value }| (value.return_type, name.primary_name))
        .collect::<Vec<_>>();
    let descriptor = check!(
        get_struct_memory_layout(&fields_for_layout[..]),
        return err(warnings, errors),
        warnings,
        errors
    );

    let total_size = descriptor.total_size();

    asm_buf.push(Op::new_comment(format!(
        "{} struct initialization",
        struct_name.primary_name
    )));

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
    for TypedStructExpressionField { name, value } in fields {
        // evaluate the expression
        let return_register = register_sequencer.next();
        let value_stack_size: u64 = match resolve_type(value.return_type, &name.span) {
            Ok(o) => match o.stack_size_of(&name.span) {
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
                    value_stack_size, name.primary_name
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
                comment: format!(
                    "cp type size {} for field {}",
                    value_stack_size, name.primary_name
                ),
            });
        } else {
            asm_buf.push(Op::write_register_to_memory(
                struct_beginning_pointer.clone(),
                return_register,
                VirtualImmediate12::new_unchecked(offset, "the whole struct is less than 12 bits so every individual field should be as well."),
                name.span.clone(),
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
