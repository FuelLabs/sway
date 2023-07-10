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

        // if combine_binary_op(context, &function) {
        //     modified = true;
        //     continue;
        // }

        // if combine_unary_op(context, &function) {
        //     modified = true;
        //     continue;
        // }

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

fn combine_binary_op(context: &mut Context, function: &Function) -> bool {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction::BinaryOp { op, arg1, arg2 })
                    if arg1.is_constant(context) && arg2.is_constant(context) =>
                {
                    let val1 = arg1.get_constant(context).unwrap();
                    let val2 = arg2.get_constant(context).unwrap();
                    match op {
                        crate::BinaryOpKind::Add => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                l.checked_add(*r).map(|v| (inst_val, block, v))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Sub => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                l.checked_sub(*r).map(|v| (inst_val, block, v))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Mul => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                l.checked_mul(*r).map(|v| (inst_val, block, v))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Div => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                l.checked_div(*r).map(|v| (inst_val, block, v))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::And => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                Some((inst_val, block, l & r))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Or => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                Some((inst_val, block, l | r))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Xor => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                Some((inst_val, block, l ^ r))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Mod => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                Some((inst_val, block, l % r))
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Rsh => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => u32::try_from(*r)
                                .ok()
                                .and_then(|r| l.checked_shr(r))
                                .map(|v| (inst_val, block, v)),
                            _ => None,
                        },
                        crate::BinaryOpKind::Lsh => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => u32::try_from(*r)
                                .ok()
                                .and_then(|r| l.checked_shl(r))
                                .map(|v| (inst_val, block, v)),
                            _ => None,
                        },
                    }
                }
                _ => None,
            },
        );

    // Replace this binary op instruction with a constant.
    candidate.map_or(false, |(inst_val, block, cn_replace)| {
        let constant = Constant::new_uint(context, 64, cn_replace);
        inst_val.replace(context, ValueDatum::Constant(constant));
        block.remove_instruction(context, inst_val);
        true
    })
}

fn combine_unary_op(context: &mut Context, function: &Function) -> bool {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction::UnaryOp { op, arg })
                    if arg.is_constant(context) =>
                {
                    let val = arg.get_constant(context).unwrap();
                    match op {
                        crate::UnaryOpKind::Not => match &val.value {
                            ConstantValue::Uint(v) => Some((inst_val, block, !v)),
                            _ => None,
                        },
                    }
                }
                _ => None,
            },
        );

    // Replace this binary op instruction with a constant.
    candidate.map_or(false, |(inst_val, block, cn_replace)| {
        let constant = Constant::new_uint(context, 64, cn_replace);
        inst_val.replace(context, ValueDatum::Constant(constant));
        block.remove_instruction(context, inst_val);
        true
    })
}

#[cfg(test)]
mod tests {
    use crate::optimize::tests::*;

    fn assert_unary_op_is_optimized(opcode: &str, v: &str, result: &str) {
        assert_is_optimized(
            &["constcombine"],
            &format!(
                "
        entry fn main() -> u64 {{
            entry():
            v = const u64 {v}
            result = {opcode} v, !0
            ret u64 result
        }}
    "
            ),
            [format!("v0 = const u64 {result}").as_str()],
        );
    }

    fn assert_binary_op_is_optimized(opcode: &str, l: &str, r: &str, result: &str) {
        assert_is_optimized(
            &["constcombine"],
            &format!(
                "
        entry fn main() -> u64 {{
            entry():
            l = const u64 {l}
            r = const u64 {r}
            result = {opcode} l, r, !0
            ret u64 result
        }}
    "
            ),
            [format!("v0 = const u64 {result}").as_str()],
        );
    }

    fn assert_binary_op_is_not_optimized(opcode: &str, l: &str, r: &str) {
        assert_is_not_optimized(
            &["constcombine"],
            &format!(
                "
        entry fn main() -> u64 {{
            entry():
            l = const u64 {l}
            r = const u64 {r}
            result = {opcode} l, r, !0
            ret u64 result
        }}
    "
            ),
        );
    }

    #[test]
    fn unary_op_are_optimized() {
        assert_unary_op_is_optimized("not", &u64::MAX.to_string(), "0");
    }

    #[test]
    fn binary_op_are_optimized() {
        assert_binary_op_is_optimized("add", "1", "1", "2");
        assert_binary_op_is_optimized("sub", "1", "1", "0");
        assert_binary_op_is_optimized("mul", "2", "2", "4");
        assert_binary_op_is_optimized("div", "10", "5", "2");
        assert_binary_op_is_optimized("mod", "12", "5", "2");
        assert_binary_op_is_optimized("rsh", "16", "1", "8");
        assert_binary_op_is_optimized("lsh", "16", "1", "32");

        assert_binary_op_is_optimized(
            "and",
            &0x00FFF.to_string(),
            &0xFFF00.to_string(),
            &0xF00.to_string(),
        );
        assert_binary_op_is_optimized(
            "or",
            &0x00FFF.to_string(),
            &0xFFF00.to_string(),
            &0xFFFFF.to_string(),
        );

        assert_binary_op_is_optimized(
            "xor",
            &0x00FFF.to_string(),
            &0xFFF00.to_string(),
            &0xFF0FF.to_string(),
        );
    }

    #[test]
    fn binary_op_are_not_optimized() {
        assert_binary_op_is_not_optimized("add", &u64::MAX.to_string(), "1");
        assert_binary_op_is_not_optimized("sub", "0", "1");
        assert_binary_op_is_not_optimized("mul", &u64::MAX.to_string(), "2");
        assert_binary_op_is_not_optimized("div", "1", "0");

        assert_binary_op_is_not_optimized("rsh", "1", "64");
        assert_binary_op_is_not_optimized("lsh", "1", "64");
    }

    #[test]
    fn ok_chain_optimization() {
        // Unary operator

        // `sub 1` is used to guarantee that the assert string is unique
        assert_is_optimized(
            &["constcombine"],
            "
        entry fn main() -> u64 {
            entry():
            a = const u64 18446744073709551615
            b = not a, !0
            c = not b, !0
            d = const u64 1
            result = sub c, d, !0
            ret u64 result
        }
    ",
            ["const u64 18446744073709551614"],
        );

        // Binary Operators
        assert_is_optimized(
            &["constcombine"],
            "
        entry fn main() -> u64 {
            entry():
            l0 = const u64 1
            r0 = const u64 2
            l1 = add l0, r0, !0
            r1 = const u64 3
            result = add l1, r1, !0
            ret u64 result
        }
    ",
            ["const u64 6"],
        );
    }
}
