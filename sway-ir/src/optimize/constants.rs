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
use num_traits::ops::checked::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use std::ops::{Shl, Shr};

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

        if combine_binary_op(context, &function) {
            modified = true;
            continue;
        }

        if combine_unary_op(context, &function) {
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
                        Predicate::Equal => Some((inst_val, block, val1.eq(context, val2))),
                        Predicate::GreaterThan => {
                            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) =
                                (&val1.value, &val2.value)
                            else {
                                unreachable!(
                                    "Type checker allowed non integer value for GreaterThan"
                                )
                            };
                            Some((inst_val, block, val1 > val2))
                        }
                        Predicate::LessThan => {
                            let (ConstantValue::Uint(val1), ConstantValue::Uint(val2)) =
                                (&val1.value, &val2.value)
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
                    let v = match op {
                        crate::BinaryOpKind::Add => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => l.checked_add(r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Sub => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => l.checked_sub(r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Mul => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => l.checked_mul(r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Div => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => l.checked_div(r),
                            _ => None,
                        },
                        crate::BinaryOpKind::And => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => Some(l & r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Or => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => Some(l | r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Xor => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => Some(l ^ r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Mod => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => Some(l % r),
                            _ => None,
                        },
                        crate::BinaryOpKind::Rsh => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                u32::try_from(r).ok().and_then(|r| {
                                    // Copy rust checked_shr behaviour
                                    let width = val1.ty.get_uint_width(context)? as u32;
                                    if r >= width {
                                        None
                                    } else {
                                        Some(l.shr(r))
                                    }
                                })
                            }
                            _ => None,
                        },
                        crate::BinaryOpKind::Lsh => match (&val1.value, &val2.value) {
                            (ConstantValue::Uint(l), ConstantValue::Uint(r)) => {
                                u32::try_from(r).ok().and_then(|r| {
                                    // Copy rust checked_shr behaviour
                                    let width = val1.ty.get_uint_width(context)? as u32;
                                    if r >= width {
                                        None
                                    } else {
                                        let mut new_value = l.shl(r);
                                        for i in width..(new_value.bits() as u32) {
                                            new_value.set_bit(i as u64, false);
                                        }
                                        Some(new_value)
                                    }
                                })
                            }
                            _ => None,
                        },
                    };

                    v.and_then(|v| {
                        let width = val1.ty.get_uint_width(&context).unwrap() as u64;
                        if v.bits() > width {
                            None
                        } else {
                            Some((
                                inst_val,
                                block,
                                Constant {
                                    ty: val1.ty,
                                    value: ConstantValue::Uint(v),
                                },
                            ))
                        }
                    })
                }
                _ => None,
            },
        );

    // Replace this binary op instruction with a constant.
    candidate.map_or(false, |(inst_val, block, new_value)| {
        inst_val.replace(context, ValueDatum::Constant(new_value));
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
                            ConstantValue::Uint(v) => {
                                val.ty.get_uint_width(context).and_then(|width| {
                                    let max = match width {
                                        8 => u8::MAX as u64,
                                        16 => u16::MAX as u64,
                                        32 => u32::MAX as u64,
                                        64 => u64::MAX,
                                        256 => todo!(),
                                        _ => return None,
                                    };
                                    let v = u64::try_from(v).ok()?;
                                    let new_value = (!v) & max;
                                    Some((
                                        inst_val,
                                        block,
                                        Constant {
                                            ty: val.ty,
                                            value: ConstantValue::Uint(new_value.into()),
                                        },
                                    ))
                                })
                            }
                            _ => None,
                        },
                    }
                }
                _ => None,
            },
        );

    // Replace this unary op instruction with a constant.
    candidate.map_or(false, |(inst_val, block, new_value)| {
        inst_val.replace(context, ValueDatum::Constant(new_value));
        block.remove_instruction(context, inst_val);
        true
    })
}

#[cfg(test)]
mod tests {
    use crate::optimize::tests::*;

    fn assert_operator(opcode: &str, t: &str, l: &str, r: Option<&str>, result: Option<&str>) {
        let expected = result.map(|result| format!("v0 = const {t} {result}"));
        let expected = expected.as_ref().map(|x| vec![x.as_str()]);
        let body = format!(
            "
    entry fn main() -> {t} {{
        entry():
        l = const {t} {l}
        {r_inst}
        result = {opcode} l, {result_inst} !0
        ret {t} result
    }}
",
            r_inst = r.map_or("".into(), |r| format!("r = const {t} {r}")),
            result_inst = r.map_or("", |_| " r,")
        );
        assert_optimization(&["constcombine"], &body, expected);
    }

    #[test]
    fn unary_op_are_optimized() {
        assert_operator("not", "u64", &u64::MAX.to_string(), None, Some("0"));
    }

    #[test]
    fn binary_op_are_optimized() {
        assert_operator("add", "u64", "1", Some("1"), Some("2"));
        assert_operator("sub", "u64", "1", Some("1"), Some("0"));
        assert_operator("mul", "u64", "2", Some("2"), Some("4"));
        assert_operator("div", "u64", "10", Some("5"), Some("2"));
        assert_operator("mod", "u64", "12", Some("5"), Some("2"));
        assert_operator("rsh", "u64", "16", Some("1"), Some("8"));
        assert_operator("lsh", "u64", "16", Some("1"), Some("32"));

        assert_operator(
            "and",
            "u64",
            &0x00FFF.to_string(),
            Some(&0xFFF00.to_string()),
            Some(&0xF00.to_string()),
        );
        assert_operator(
            "or",
            "u64",
            &0x00FFF.to_string(),
            Some(&0xFFF00.to_string()),
            Some(&0xFFFFF.to_string()),
        );

        assert_operator(
            "xor",
            "u64",
            &0x00FFF.to_string(),
            Some(&0xFFF00.to_string()),
            Some(&0xFF0FF.to_string()),
        );

        // u256
        assert_operator("add", "u256", "1", Some("1"), Some("2"));
        assert_operator("sub", "u256", "1", Some("1"), Some("0"));
        assert_operator("mul", "u256", "2", Some("2"), Some("4"));
        assert_operator("div", "u256", "10", Some("5"), Some("2"));
        // assert_operator("mod", "u64", "12", Some("5"), Some("2"));
        assert_operator("rsh", "u256", "16", Some("1"), Some("8"));
        assert_operator("lsh", "u256", "16", Some("1"), Some("32"));
    }

    #[test]
    fn binary_op_are_not_optimized() {
        assert_operator("add", "u64", &u64::MAX.to_string(), Some("1"), None);
        assert_operator("sub", "u64", "0", Some("1"), None);
        assert_operator("mul", "u64", &u64::MAX.to_string(), Some("2"), None);
        assert_operator("div", "u64", "1", Some("0"), None);

        assert_operator("rsh", "u64", "1", Some("64"), None);
        assert_operator("lsh", "u64", "1", Some("64"), None);
    }

    #[test]
    fn ok_chain_optimization() {
        // Unary operator

        // `sub 1` is used to guarantee that the assert string is unique
        assert_optimization(
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
            Some(["const u64 18446744073709551614"]),
        );

        // Binary Operators
        assert_optimization(
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
            Some(["const u64 6"]),
        );
    }
}
