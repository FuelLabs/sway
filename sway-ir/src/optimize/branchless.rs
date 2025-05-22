use sway_features::ExperimentalFeatures;

use crate::{AnalysisResults, Block, BranchToWithArgs, Constant, ConstantContent, Context, Function, InstOp, IrError, Pass, PassMutability, ScopedPass, Value, ValueDatum};


pub const BRANCHLESS_NAME: &str = "branchless";

pub fn create_branchless() -> Pass {
    Pass {
        name: BRANCHLESS_NAME,
        descr: "Branchless",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(branchless)),
    }
}

// check if a block simple calls another block with a u64
fn is_block_simple_integer<'a>(context: &'a Context, branch: &BranchToWithArgs) -> Option<(&'a Constant, &'a Block)> {
    let b = &context.blocks[branch.block.0];

    if b.instructions.len() > 1 {
        return None;
    }

    let v = &b.instructions[0];
    let v = &context.values[v.0];
    match &v.value {
        crate::ValueDatum::Instruction(i) => match &i.op {
            InstOp::Branch(branch) => {
                if branch.args.len() != 1 {
                    return None;
                }
                
                let arg0 = &context.values[branch.args[0].0];
                match &arg0.value {
                    crate::ValueDatum::Constant(constant) => Some((constant, &branch.block)),
                    _ => None,
                }
            },
            _ => None,
        },
        _ => None,
    }
}

fn find_cbr(context: &mut Context, function: Function) -> Option<(Block, Value, Block, Value, Constant, Constant)> {
    for (block, value) in function.instruction_iter(context) {
        match &context.values[value.0].value {
            ValueDatum::Argument(_) => {},
            ValueDatum::Constant(_) => {},
            ValueDatum::Instruction(instruction) => {
                match &instruction.op {
                    InstOp::ConditionalBranch { cond_value, true_block, false_block } => {
                        let target_block_true = is_block_simple_integer(context, &true_block);
                        let target_block_false = is_block_simple_integer(context, &false_block);

                        // both branches call the same block
                        match (target_block_true, target_block_false) {
                            (Some((constant_true, target_block_true)), Some((constant_false, target_block_false))) if target_block_true == target_block_false => {
                                return Some((block, value, *target_block_true, *cond_value, *constant_true, *constant_false));
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            },
        };
    }

    None
}

pub fn branchless(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    return Ok(false);

    loop {
        if let Some((block, instr_val, target_block, cond_value, constant_true, constant_false)) = find_cbr(context, function) {
            block.remove_instruction(context, instr_val);

            let one = ConstantContent::new_uint(context, 64, 1);
            let one = Constant::unique(context, one);
            let one = Value::new_constant(context, one);
            let a = Value::new_constant(context, constant_true);
            let b = Value::new_constant(context, constant_false);
            
            // c is a boolean (1 or 0)
            // Can we use predication?
            // x = c * a + (1 − c) * b
            let c_times_a = Value::new_instruction(context, block, InstOp::BinaryOp { op: crate::BinaryOpKind::Mul, arg1: cond_value, arg2: a });
            let one_minus_c = Value::new_instruction(context, block, InstOp::BinaryOp { op: crate::BinaryOpKind::Sub, arg1: one, arg2: cond_value });
            let one_minus_c_times_b = Value::new_instruction(context, block, InstOp::BinaryOp { op: crate::BinaryOpKind::Mul, arg1: one_minus_c, arg2: b });
            let x = Value::new_instruction(context, block, InstOp::BinaryOp { op: crate::BinaryOpKind::Add, arg1: c_times_a, arg2: one_minus_c_times_b });

            block.insert_instructions_after(context, cond_value, [c_times_a, one_minus_c, one_minus_c_times_b, x]);

            let call_target_block = Value::new_instruction(context, block, InstOp::Branch(BranchToWithArgs { 
                block: target_block,
                args: vec![x]
            }));

            let block = &mut context.blocks[block.0];
            block.instructions.push(call_target_block);

            modified = true;
        } else {
            break;
        }
    }

    eprintln!("{}", context.to_string());

    Ok(modified)
}

#[cfg(test)]
mod tests {
    use crate::tests::assert_optimization;
    use super::BRANCHLESS_NAME;

    #[test]
    fn branchless_optimized() {
        let before_optimization = format!(
            "
    fn main(baba !68: u64) -> u64, !71 {{
        entry(baba: u64):
        v0 = const u64 0, !72
        cbr v0, block0(), block1(), !73

        block0():
        v2 = const u64 1, !76
        br block2(v2)

        block1():
        v3 = const u64 2, !77
        br block2(v3)

        block2(v4: u64):
        ret u64 v4
    }}
",
        );
        assert_optimization(&[BRANCHLESS_NAME], &before_optimization, Some(["const u64 1, !76"]));
    }
}