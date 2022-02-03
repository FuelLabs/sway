//! Optimization passes for manipulating constant values.
//!
//! - combining - compile time evaluation of constant expressions.
//!   - combine insert_values - reduce expressions which insert a constant value into a constant
//!     struct.

use crate::{
    constant::{Constant, ConstantValue},
    context::Context,
    function::Function,
    instruction::Instruction,
    value::{Value, ValueContent},
};

/// Find constant expressions which can be reduced to fewer opterations.
pub fn combine_constants(context: &mut Context, function: &Function) -> Result<bool, String> {
    let mut modified = false;
    loop {
        if combine_const_insert_values(context, function) {
            modified = true;
            continue;
        }

        // Other passes here... always continue to the top if pass returns true.
        break;
    }
    Ok(modified)
}

fn combine_const_insert_values(context: &mut Context, function: &Function) -> bool {
    // Find a candidate `insert_value` instruction.
    let candidate = function
        .instruction_iter(context)
        .find_map(|(block, ins_val)| {
            match &context.values[ins_val.0] {
                // We only want inject this constant value into a constant aggregate declaration,
                // not another `insert_value` instruction.
                //
                // We *could* trace back to the original aggregate through other `insert_value`s
                // but we'd have to be careful that this constant value isn't clobbered by the
                // chain.  It's simpler to just combine the instruction which modifies the
                // aggregate directly and then to iterate.
                ValueContent::Instruction(Instruction::InsertValue {
                    aggregate,
                    ty: _,
                    value,
                    indices,
                }) if value.is_constant(context)
                    && matches!(
                        &context.values[aggregate.0],
                        ValueContent::Constant(Constant {
                            value: ConstantValue::Struct(_),
                            ..
                        }),
                    ) =>
                {
                    Some((block, ins_val, *aggregate, *value, indices.clone()))
                }
                _otherwise => None,
            }
        });

    if let Some((block, ins_val, aggregate, const_val, indices)) = candidate {
        // OK, here we have an `insert_value` of a constant directly into a constant
        // aggregate.  We want to replace the constant aggregate with an updated one.
        let new_aggregate =
            combine_const_aggregate_field(context, function, aggregate, const_val, &indices);

        // Replace uses of the `insert_value` instruction with the new aggregate.
        function.replace_value(context, ins_val, new_aggregate, None);

        // Remove the `insert_value` instruction.
        block.remove_instruction(context, ins_val);

        // Let's return now, since our iterator may get confused and let the pass
        // iterate further itself.
        return true;
    }

    false
}

fn combine_const_aggregate_field(
    context: &mut Context,
    function: &Function,
    aggregate: Value,
    const_value: Value,
    indices: &[u64],
) -> Value {
    // Create a copy of the aggregate constant and inserted value.
    let mut new_aggregate = match &context.values[aggregate.0] {
        ValueContent::Constant(c) => c.clone(),
        _otherwise => {
            unreachable!("BUG! Invalid aggregate parameter to combine_const_insert_value()")
        }
    };
    let const_value = match &context.values[const_value.0] {
        ValueContent::Constant(c) => c.clone(),
        _otherwise => {
            unreachable!("BUG! Invalid const_value parameter to combine_const_insert_value()")
        }
    };

    // Update the new aggregate with the constant field, based in the indices.
    inject_constant_into_aggregate(&mut new_aggregate, const_value, indices);

    // Replace the old aggregate with the new aggregate.
    let new_aggregate_value = Value::new_constant(context, new_aggregate);
    function.replace_value(context, aggregate, new_aggregate_value, None);

    // Remove the old aggregate from the context.
    //
    // OR NOT!  This is too dangerous unless we can
    // guarantee it has no uses, which is something we should implement eventually.  For now, in
    // this case it shouldn't matter if we leave it, even if it's not used.
    //
    // TODO: context.values.remove(aggregate.0);

    new_aggregate_value
}

fn inject_constant_into_aggregate(aggregate: &mut Constant, value: Constant, indices: &[u64]) {
    if indices.is_empty() {
        *aggregate = value;
    } else {
        match &mut aggregate.value {
            ConstantValue::Struct(fields) => inject_constant_into_aggregate(
                &mut fields[indices[0] as usize],
                value,
                &indices[1..],
            ),
            _otherwise => {
                unreachable!("Bug! Invalid aggregate parameter to inject_constant_into_aggregate()")
            }
        }
    }
}
