//! Optimization passes for manipulating constant values.

use crate::{
    constant::{ConstantContent, ConstantValue},
    context::Context,
    error::IrError,
    function::Function,
    instruction::InstOp,
    value::ValueDatum,
    AnalysisResults, BranchToWithArgs, Constant, Instruction, Pass, PassMutability, Predicate,
    ScopedPass,
};
use rustc_hash::FxHashMap;

pub const CONST_FOLDING_NAME: &str = "const-folding";

pub fn create_const_folding_pass() -> Pass {
    Pass {
        name: CONST_FOLDING_NAME,
        descr: "Constant folding",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(fold_constants)),
    }
}

/// Find constant expressions which can be reduced to fewer operations.
pub fn fold_constants(
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

        if remove_useless_binary_op(context, &function) {
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
                ValueDatum::Instruction(Instruction {
                    op:
                        InstOp::ConditionalBranch {
                            cond_value,
                            true_block,
                            false_block,
                        },
                    ..
                }) if cond_value.is_constant(context) => {
                    match &cond_value
                        .get_constant(context)
                        .unwrap()
                        .get_content(context)
                        .value
                    {
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
            // `no_more_dest` will no longer have from_block as a predecessor.
            no_more_dest.remove_pred(context, &from_block);
            // Although our cbr already branched to `dest`, in case
            // `no_more_dest` and `dest` are the same, we'll need to re-add
            // `from_block` as a predecessor for `dest`.
            dest.block.add_pred(context, &from_block);
            cbr.replace(
                context,
                ValueDatum::Instruction(Instruction {
                    op: InstOp::Branch(dest),
                    parent: cbr.get_instruction(context).unwrap().parent,
                }),
            );
            Ok(true)
        },
    )
}

fn combine_cmp(context: &mut Context, function: &Function) -> bool {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction {
                    op: InstOp::Cmp(pred, val1, val2),
                    ..
                }) if val1.is_constant(context) && val2.is_constant(context) => {
                    let val1 = val1.get_constant(context).unwrap();
                    let val2 = val2.get_constant(context).unwrap();

                    use ConstantValue::*;
                    match pred {
                        Predicate::Equal => Some((inst_val, block, val1 == val2)),
                        Predicate::GreaterThan => {
                            let r = match (
                                &val1.get_content(context).value,
                                &val2.get_content(context).value,
                            ) {
                                (Uint(val1), Uint(val2)) => val1 > val2,
                                (U256(val1), U256(val2)) => val1 > val2,
                                (B256(val1), B256(val2)) => val1 > val2,
                                _ => {
                                    unreachable!(
                                        "Type checker allowed non integer value for GreaterThan"
                                    )
                                }
                            };
                            Some((inst_val, block, r))
                        }
                        Predicate::LessThan => {
                            let r = match (
                                &val1.get_content(context).value,
                                &val2.get_content(context).value,
                            ) {
                                (Uint(val1), Uint(val2)) => val1 < val2,
                                (U256(val1), U256(val2)) => val1 < val2,
                                (B256(val1), B256(val2)) => val1 < val2,
                                _ => {
                                    unreachable!(
                                        "Type checker allowed non integer value for GreaterThan"
                                    )
                                }
                            };
                            Some((inst_val, block, r))
                        }
                    }
                }
                _ => None,
            },
        );

    candidate.is_some_and(|(inst_val, block, cn_replace)| {
        let const_content = ConstantContent::new_bool(context, cn_replace);
        let constant = crate::Constant::unique(context, const_content);
        // Replace this `cmp` instruction with a constant.
        inst_val.replace(context, ValueDatum::Constant(constant));
        block.remove_instruction(context, inst_val);
        true
    })
}

fn combine_binary_op(context: &mut Context, function: &Function) -> bool {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction {
                    op: InstOp::BinaryOp { op, arg1, arg2 },
                    ..
                }) if arg1.is_constant(context) && arg2.is_constant(context) => {
                    let val1 = arg1.get_constant(context).unwrap().get_content(context);
                    let val2 = arg2.get_constant(context).unwrap().get_content(context);
                    use crate::BinaryOpKind::*;
                    use ConstantValue::*;
                    let v = match (op, &val1.value, &val2.value) {
                        (Add, Uint(l), Uint(r)) => l.checked_add(*r).map(Uint),
                        (Add, U256(l), U256(r)) => l.checked_add(r).map(U256),

                        (Sub, Uint(l), Uint(r)) => l.checked_sub(*r).map(Uint),
                        (Sub, U256(l), U256(r)) => l.checked_sub(r).map(U256),

                        (Mul, Uint(l), Uint(r)) => l.checked_mul(*r).map(Uint),
                        (Mul, U256(l), U256(r)) => l.checked_mul(r).map(U256),

                        (Div, Uint(l), Uint(r)) => l.checked_div(*r).map(Uint),
                        (Div, U256(l), U256(r)) => l.checked_div(r).map(U256),

                        (And, Uint(l), Uint(r)) => Some(Uint(l & r)),
                        (And, U256(l), U256(r)) => Some(U256(l & r)),

                        (Or, Uint(l), Uint(r)) => Some(Uint(l | r)),
                        (Or, U256(l), U256(r)) => Some(U256(l | r)),

                        (Xor, Uint(l), Uint(r)) => Some(Uint(l ^ r)),
                        (Xor, U256(l), U256(r)) => Some(U256(l ^ r)),

                        (Mod, Uint(l), Uint(r)) => l.checked_rem(*r).map(Uint),
                        (Mod, U256(l), U256(r)) => l.checked_rem(r).map(U256),

                        (Rsh, Uint(l), Uint(r)) => u32::try_from(*r)
                            .ok()
                            .and_then(|r| l.checked_shr(r).map(Uint)),
                        (Rsh, U256(l), Uint(r)) => Some(U256(l.shr(r))),

                        (Lsh, Uint(l), Uint(r)) => u32::try_from(*r)
                            .ok()
                            .and_then(|r| l.checked_shl(r).map(Uint)),
                        (Lsh, U256(l), Uint(r)) => l.checked_shl(r).map(U256),
                        _ => None,
                    };
                    v.map(|value| (inst_val, block, ConstantContent { ty: val1.ty, value }))
                }
                _ => None,
            },
        );

    // Replace this binary op instruction with a constant.
    candidate.is_some_and(|(inst_val, block, new_value)| {
        let new_value = Constant::unique(context, new_value);
        inst_val.replace(context, ValueDatum::Constant(new_value));
        block.remove_instruction(context, inst_val);
        true
    })
}

fn remove_useless_binary_op(context: &mut Context, function: &Function) -> bool {
    let candidate =
        function
            .instruction_iter(context)
            .find_map(
                |(block, candidate)| match &context.values[candidate.0].value {
                    ValueDatum::Instruction(Instruction {
                        op: InstOp::BinaryOp { op, arg1, arg2 },
                        ..
                    }) if arg1.is_constant(context) || arg2.is_constant(context) => {
                        let val1 = arg1
                            .get_constant(context)
                            .map(|x| &x.get_content(context).value);
                        let val2 = arg2
                            .get_constant(context)
                            .map(|x| &x.get_content(context).value);

                        use crate::BinaryOpKind::*;
                        use ConstantValue::*;
                        match (op, val1, val2) {
                            // 0 + arg2
                            (Add, Some(Uint(0)), _) => Some((block, candidate, *arg2)),
                            // arg1 + 0
                            (Add, _, Some(Uint(0))) => Some((block, candidate, *arg1)),
                            // 1 * arg2
                            (Mul, Some(Uint(1)), _) => Some((block, candidate, *arg2)),
                            // arg1 * 1
                            (Mul, _, Some(Uint(1))) => Some((block, candidate, *arg1)),
                            // arg1 / 1
                            (Div, _, Some(Uint(1))) => Some((block, candidate, *arg1)),
                            // arg1 - 0
                            (Sub, _, Some(Uint(0))) => Some((block, candidate, *arg1)),
                            _ => None,
                        }
                    }
                    _ => None,
                },
            );

    candidate.is_some_and(|(block, old_value, new_value)| {
        let replace_map = FxHashMap::from_iter([(old_value, new_value)]);
        function.replace_values(context, &replace_map, None);

        block.remove_instruction(context, old_value);
        true
    })
}

fn combine_unary_op(context: &mut Context, function: &Function) -> bool {
    let candidate = function
        .instruction_iter(context)
        .find_map(
            |(block, inst_val)| match &context.values[inst_val.0].value {
                ValueDatum::Instruction(Instruction {
                    op: InstOp::UnaryOp { op, arg },
                    ..
                }) if arg.is_constant(context) => {
                    let val = arg.get_constant(context).unwrap();
                    use crate::UnaryOpKind::*;
                    use ConstantValue::*;
                    let v = match (op, &val.get_content(context).value) {
                        (Not, Uint(v)) => val
                            .get_content(context)
                            .ty
                            .get_uint_width(context)
                            .and_then(|width| {
                                let max = match width {
                                    8 => u8::MAX as u64,
                                    16 => u16::MAX as u64,
                                    32 => u32::MAX as u64,
                                    64 => u64::MAX,
                                    _ => return None,
                                };
                                Some(Uint((!v) & max))
                            }),
                        (Not, U256(v)) => Some(U256(!v)),
                        _ => None,
                    };
                    v.map(|value| {
                        (
                            inst_val,
                            block,
                            ConstantContent {
                                ty: val.get_content(context).ty,
                                value,
                            },
                        )
                    })
                }
                _ => None,
            },
        );

    // Replace this unary op instruction with a constant.
    candidate.is_some_and(|(inst_val, block, new_value)| {
        let new_value = Constant::unique(context, new_value);
        inst_val.replace(context, ValueDatum::Constant(new_value));
        block.remove_instruction(context, inst_val);
        true
    })
}

#[cfg(test)]
mod tests {
    use crate::{optimize::tests::*, CONST_FOLDING_NAME};

    fn assert_operator(t: &str, opcode: &str, l: &str, r: Option<&str>, result: Option<&str>) {
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
        assert_optimization(&[CONST_FOLDING_NAME], &body, expected);
    }

    #[test]
    fn unary_op_are_optimized() {
        assert_operator("u64", "not", &u64::MAX.to_string(), None, Some("0"));
    }

    #[test]
    fn binary_op_are_optimized() {
        // u64
        assert_operator("u64", "add", "1", Some("1"), Some("2"));
        assert_operator("u64", "sub", "1", Some("1"), Some("0"));
        assert_operator("u64", "mul", "2", Some("2"), Some("4"));
        assert_operator("u64", "div", "10", Some("5"), Some("2"));
        assert_operator("u64", "mod", "12", Some("5"), Some("2"));
        assert_operator("u64", "rsh", "16", Some("1"), Some("8"));
        assert_operator("u64", "lsh", "16", Some("1"), Some("32"));

        assert_operator(
            "u64",
            "and",
            &0x00FFF.to_string(),
            Some(&0xFFF00.to_string()),
            Some(&0xF00.to_string()),
        );
        assert_operator(
            "u64",
            "or",
            &0x00FFF.to_string(),
            Some(&0xFFF00.to_string()),
            Some(&0xFFFFF.to_string()),
        );

        assert_operator(
            "u64",
            "xor",
            &0x00FFF.to_string(),
            Some(&0xFFF00.to_string()),
            Some(&0xFF0FF.to_string()),
        );
    }

    #[test]
    fn binary_op_are_not_optimized() {
        assert_operator("u64", "add", &u64::MAX.to_string(), Some("1"), None);
        assert_operator("u64", "sub", "0", Some("1"), None);
        assert_operator("u64", "mul", &u64::MAX.to_string(), Some("2"), None);
        assert_operator("u64", "div", "1", Some("0"), None);
        assert_operator("u64", "mod", "1", Some("0"), None);

        assert_operator("u64", "rsh", "1", Some("64"), None);
        assert_operator("u64", "lsh", "1", Some("64"), None);
    }

    #[test]
    fn ok_chain_optimization() {
        // Unary operator

        // `sub 1` is used to guarantee that the assert string is unique
        assert_optimization(
            &[CONST_FOLDING_NAME],
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
            &[CONST_FOLDING_NAME],
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

    #[test]
    fn ok_remove_useless_mul() {
        assert_optimization(
            &[CONST_FOLDING_NAME],
            "entry fn main() -> u64 {
                local u64 LOCAL
            entry():
                zero = const u64 0, !0
                one = const u64 1, !0
                l_ptr = get_local __ptr u64, LOCAL, !0
                l = load l_ptr, !0
                result1 = mul l, one, !0
                result2 = mul one, result1, !0
                result3 = add result2, zero, !0
                result4 = add zero, result3, !0
                result5 = div result4, one, !0
                result6 = sub result5, zero, !0
                ret u64 result6, !0
         }",
            Some([
                "v0 = get_local __ptr u64, LOCAL",
                "v1 = load v0",
                "ret u64 v1",
            ]),
        );
    }
}
