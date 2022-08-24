//! Optimization passes for manipulating constant values.
//!
//! - combining - compile time evaluation of constant expressions.
//!   - combine insert_values - reduce expressions which insert a constant value into a constant
//!     struct.

use crate::{
    constant::{Constant, ConstantValue},
    context::Context,
    error::IrError,
    function::Function,
    instruction::Instruction,
    value::{Value, ValueContent, ValueDatum},
    Predicate,
};

/// Find constant expressions which can be reduced to fewer opterations.
pub fn combine_constants(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut modified = false;
    loop {
        if combine_const_insert_values(context, function) {
            modified = true;
            continue;
        }

        if fold_cmp(context, function) {
            modified = true;
            continue;
        }

        if fold_cbr(context, function)? {
            modified = true;
            continue;
        }

        // Other passes here... always continue to the top if pass returns true.
        break;
    }

    Ok(modified)
}

fn fold_cbr(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(in_block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                }) if cond_value.is_constant(context) => {
                    match cond_value.get_constant(context).unwrap().value {
                        ConstantValue::Bool(true) => {
                            Some(Ok((inst_val, in_block, *true_block, *false_block)))
                        }
                        ConstantValue::Bool(false) => {
                            Some(Ok((inst_val, in_block, *false_block, *true_block)))
                        }
                        _ => Some(Err(IrError::VerifyConditionExprNotABool)),
                    }
                }
                _ => None,
            },
        );

    match candidate {
        Some(res) => {
            let (cbr, from_block, dest, no_more_dest) = res?;
            no_more_dest.remove_phi_val_coming_from(context, &from_block);
            context.values[cbr.0].value = ValueDatum::Instruction(Instruction::Branch(dest));
            Ok(true)
        }
        None => Ok(false),
    }
}

fn fold_cmp(context: &mut Context, function: &Function) -> bool {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction::Cmp(pred, val1, val2))
                    if val1.is_constant(context) && val2.is_constant(context) =>
                {
                    let val1 = val1.get_constant(context).unwrap();
                    let val2 = val2.get_constant(context).unwrap();
                    match pred {
                        Predicate::Equal => {
                            if val1.eq(context, val2) {
                                Some((inst_val, block, true))
                            } else {
                                Some((inst_val, block, false))
                            }
                        }
                    }
                }
                _ => None,
            },
        );

    candidate.map_or(false, |(inst_val, block, cn_replace)| {
        let new_val = Value::new_constant(context, Constant::new_bool(cn_replace));
        // Replace uses of this `cmp` instruction with a constant
        function.replace_value(context, inst_val, new_val, None);
        // Remove the `cmp` instruction.
        block.remove_instruction(context, inst_val);
        true
    })
}

fn combine_const_insert_values(context: &mut Context, function: &Function) -> bool {
    // Find a candidate `insert_value` instruction.
    let candidate = function
        .instruction_iter(context)
        .find_map(|(block, ins_val)| {
            match &context.values[ins_val.0].value {
                // We only want inject this constant value into a constant aggregate declaration,
                // not another `insert_value` instruction.
                //
                // We *could* trace back to the original aggregate through other `insert_value`s
                // but we'd have to be careful that this constant value isn't clobbered by the
                // chain.  It's simpler to just combine the instruction which modifies the
                // aggregate directly and then to iterate.
                ValueDatum::Instruction(Instruction::InsertValue {
                    aggregate,
                    ty: _,
                    value,
                    indices,
                }) if value.is_constant(context)
                    && matches!(
                        &context.values[aggregate.0].value,
                        ValueDatum::Constant(Constant {
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
        // OK, here we have an `insert_value` of a constant directly into a constant aggregate.  We
        // want to replace the constant aggregate with an updated one.
        let new_aggregate = combine_const_aggregate_field(context, aggregate, const_val, &indices);

        // Replace uses of the `insert_value` instruction with the new aggregate.
        function.replace_value(context, ins_val, new_aggregate, None);

        // Remove the `insert_value` instruction.
        block.remove_instruction(context, ins_val);

        // Let's return now, since our iterator may get confused and let the pass iterate further
        // itself.
        return true;
    }

    false
}

fn combine_const_aggregate_field(
    context: &mut Context,
    aggregate: Value,
    const_value: Value,
    indices: &[u64],
) -> Value {
    // Create a copy of the aggregate constant and inserted value.
    let (mut new_aggregate, metadata) = match &context.values[aggregate.0] {
        ValueContent {
            value: ValueDatum::Constant(c),
            metadata,
        } => (c.clone(), *metadata),
        _otherwise => {
            unreachable!("BUG! Invalid aggregate parameter to combine_const_insert_value()")
        }
    };
    let const_value = match &context.values[const_value.0].value {
        ValueDatum::Constant(c) => c.clone(),
        _otherwise => {
            unreachable!("BUG! Invalid const_value parameter to combine_const_insert_value()")
        }
    };

    // Update the new aggregate with the constant field, based in the indices.
    inject_constant_into_aggregate(&mut new_aggregate, const_value, indices);

    // NOTE: Previous versions of this pass were trying to clean up after themselves, by replacing
    // the old aggregate with this new one, and/or removing the old aggregate altogether.  This is
    // too dangerous without proper checking for remaining uses, and is best left to DCE anyway.

    Value::new_constant(context, new_aggregate).add_metadatum(context, metadata)
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
