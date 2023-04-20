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
    value::ValueDatum,
    AnalysisResults, BranchToWithArgs, Pass, PassMutability, Predicate, ScopedPass,
};

pub const CONSTCOMBINE_NAME: &str = "constcombine";

pub fn create_const_combine_pass() -> Pass {
    Pass {
        name: CONSTCOMBINE_NAME,
        descr: "constant folding.",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(combine_constants)),
    }
}

/// Find constant expressions which can be reduced to fewer opterations.
pub fn combine_constants(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    loop {
        if combine_cmp(context, &function) {
            modified = true;
            continue;
        }

        if combine_cbr(context, &function)? {
            modified = true;
            continue;
        }

        // Other passes here... always continue to the top if pass returns true.
        break;
    }

    Ok(modified)
}

fn combine_cbr(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(in_block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                }) if cond_value.is_constant(context) => {
                    match &cond_value.get_constant(context).unwrap().value {
                        ConstantValue::Bool(true) => Some(Ok((
                            inst_val,
                            in_block,
                            true_block.clone(),
                            false_block.clone(),
                        ))),
                        ConstantValue::Bool(false) => Some(Ok((
                            inst_val,
                            in_block,
                            false_block.clone(),
                            true_block.clone(),
                        ))),
                        _ => Some(Err(IrError::VerifyConditionExprNotABool)),
                    }
                }
                _ => None,
            },
        )
        .transpose()?;

    candidate.map_or(
        Ok(false),
        |(
            cbr,
            from_block,
            dest,
            BranchToWithArgs {
                block: no_more_dest,
                ..
            },
        )| {
            no_more_dest.remove_pred(context, &from_block);
            cbr.replace(context, ValueDatum::Instruction(Instruction::Branch(dest)));
            Ok(true)
        },
    )
}

fn combine_cmp(context: &mut Context, function: &Function) -> bool {
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
                                Some((inst_val, block, val1.eq(context, val2)))
                        }
                        Predicate::GreaterThan => {
                            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) = (&val1.value, &val2.value)
                            else {
                                unreachable!("Type checker allowed non integer value for GreaterThan")
                            };
                            Some((inst_val, block, val1 > val2))
                        }
                        Predicate::LessThan => {
                            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) = (&val1.value, &val2.value)
                            else {
                                unreachable!("Type checker allowed non integer value for LessThan")
                            };
                            Some((inst_val, block, val1 < val2))
                        }
                    }
                }
                _ => None,
            },
        );

    candidate.map_or(false, |(inst_val, block, cn_replace)| {
        // Replace this `cmp` instruction with a constant.
        inst_val.replace(
            context,
            ValueDatum::Constant(Constant::new_bool(context, cn_replace)),
        );
        block.remove_instruction(context, inst_val);
        true
    })
}
