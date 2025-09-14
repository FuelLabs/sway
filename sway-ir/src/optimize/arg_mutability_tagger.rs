//! Tags function pointer arguments as immutable based on their usage.

use crate::{
    build_call_graph, callee_first_order, AnalysisResults, BinaryOpKind, Context,
    FuelVmInstruction, Function, InstOp, IrError, Module, Pass, PassMutability, ScopedPass, Value,
    ValueDatum,
};
use rustc_hash::{FxHashMap, FxHashSet};

pub const ARG_POINTEE_MUTABILITY_TAGGER_NAME: &str = "arg_pointee_mutability_tagger";

pub fn create_arg_pointee_mutability_tagger_pass() -> Pass {
    Pass {
        name: ARG_POINTEE_MUTABILITY_TAGGER_NAME,
        descr: "Tags function pointer arguments as immutable based on their usage",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(arg_pointee_mutability_tagger)),
    }
}

fn arg_pointee_mutability_tagger(
    context: &mut Context,
    _analysis_results: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    let fn_mutability: ArgPointeeMutabilityResult =
        compute_arg_pointee_mutability(context, module)?;

    let mut immutable_args = vec![];
    for f in module.function_iter(context) {
        assert!(fn_mutability.is_analyzed(f));
        for (arg_idx, (_arg_name, arg_val)) in f.args_iter(context).enumerate() {
            let is_immutable = matches!(
                fn_mutability.get_mutability(f, arg_idx),
                ArgPointeeMutability::Immutable
            );
            if is_immutable {
                // Tag the argument as immutable
                immutable_args.push(*arg_val);
            }
        }
    }

    let modified = !immutable_args.is_empty();

    for arg_val in immutable_args {
        let arg = arg_val
            .get_argument_mut(context)
            .expect("arg is an argument");
        arg.is_immutable = true;
    }

    Ok(modified)
}

#[derive(Debug, Clone, PartialEq)]
/// The mutability of a pointer function argument's pointee.
pub enum ArgPointeeMutability {
    Immutable,
    Mutable,
    NotAPointer,
}

/// Result of the arg pointee mutability analysis, for the arguments of each function.
#[derive(Default)]
pub struct ArgPointeeMutabilityResult(FxHashMap<Function, Vec<ArgPointeeMutability>>);

impl ArgPointeeMutabilityResult {
    /// Get the mutability of the pointee for a function argument.
    /// Panics on invalid function or argument index.
    pub fn get_mutability(&self, function: Function, arg_index: usize) -> ArgPointeeMutability {
        self.0.get(&function).unwrap()[arg_index].clone()
    }

    /// Does function have a result?
    pub fn is_analyzed(&self, function: Function) -> bool {
        self.0.contains_key(&function)
    }
}

/// For every function argument that is a pointer, determine if that function
/// may directly mutate the corresponding pointee.
/// The word "directly" is important here, as it does not consider
/// indirect mutations through contained pointers or references.
pub fn compute_arg_pointee_mutability(
    context: &Context,
    module: Module,
) -> Result<ArgPointeeMutabilityResult, IrError> {
    let cg = build_call_graph(context, &context.modules.get(module.0).unwrap().functions);
    let callee_first = callee_first_order(&cg);

    let mut res = ArgPointeeMutabilityResult::default();

    for function in callee_first.iter() {
        analyse_fn(context, *function, &mut res)?;
    }

    Ok(res)
}

// For every definition, what / where are its uses?
fn compute_def_use_chains(ctx: &Context, function: Function) -> FxHashMap<Value, FxHashSet<Value>> {
    let mut def_use: FxHashMap<Value, FxHashSet<Value>> = FxHashMap::default();

    for block in function.block_iter(ctx) {
        // The formal block arguments "use" the actual arguments that are passed to them.
        for formal_arg in block.arg_iter(ctx) {
            for pred in block.pred_iter(ctx) {
                let actual_arg = formal_arg
                    .get_argument(ctx)
                    .unwrap()
                    .get_val_coming_from(ctx, pred)
                    .unwrap();
                def_use.entry(actual_arg).or_default().insert(*formal_arg);
            }
        }

        // Instructions "use" their operands.
        for inst in block.instruction_iter(ctx) {
            for operand in inst.get_instruction(ctx).unwrap().op.get_operands() {
                def_use.entry(operand).or_default().insert(inst);
            }
        }
    }
    def_use
}

fn analyse_fn(
    ctx: &Context,
    function: Function,
    res: &mut ArgPointeeMutabilityResult,
) -> Result<(), IrError> {
    assert!(
        !res.is_analyzed(function),
        "Function {} already analyzed",
        function.get_name(ctx)
    );

    let mut has_atleast_one_pointer_arg = false;

    let mut arg_mutabilities = function
        .args_iter(ctx)
        .map(|(_arg_name, arg)| {
            if arg.get_type(ctx).is_some_and(|t| t.is_ptr(ctx)) {
                has_atleast_one_pointer_arg = true;
                // Assume that pointer arguments are not mutable by default.
                ArgPointeeMutability::Immutable
            } else {
                ArgPointeeMutability::NotAPointer
            }
        })
        .collect::<Vec<_>>();

    if !has_atleast_one_pointer_arg {
        // If there are no pointer arguments, we can skip further analysis.
        res.0.insert(function, arg_mutabilities);
        return Ok(());
    }

    let def_use = compute_def_use_chains(ctx, function);

    'analyse_next_arg: for (arg_idx, (_arg_name, arg)) in function.args_iter(ctx).enumerate() {
        if matches!(
            arg_mutabilities[arg_idx],
            ArgPointeeMutability::NotAPointer | ArgPointeeMutability::Mutable
        ) {
            continue;
        }
        // Known aliases of this argument. Also serves as a visited set.
        let mut aliases: FxHashSet<Value> = FxHashSet::default();
        let mut in_worklist = FxHashSet::default();
        let mut worklist = vec![];
        // Start with the argument value itself.
        in_worklist.insert(*arg);
        worklist.push(*arg);

        while let Some(value) = worklist.pop() {
            in_worklist.remove(&value);
            if !aliases.insert(value) {
                // If we already visited this value, skip it.
                continue;
            }

            match &ctx.values.get(value.0).unwrap().value {
                ValueDatum::Instruction(inst) => match &inst.op {
                    InstOp::ConditionalBranch { .. } | InstOp::Branch(_) => {
                        // Branch instructions do not mutate anything.
                        // They do pass arguments to the next block,
                        // but that is captured by that argument itself being
                        // considered a use.
                    }
                    InstOp::Cmp(_, _, _) | InstOp::Ret(_, _) => (),

                    InstOp::UnaryOp { .. }
                    | InstOp::BitCast(_, _)
                    | InstOp::GetLocal(_)
                    | InstOp::GetGlobal(_)
                    | InstOp::GetConfig(_, _)
                    | InstOp::GetStorageKey(_)
                    | InstOp::IntToPtr(_, _)
                    | InstOp::Nop => {
                        panic!("Pointers shouldn't be used in these instructions");
                    }
                    InstOp::BinaryOp { op, .. } => {
                        match op {
                            BinaryOpKind::Add | BinaryOpKind::Sub => {
                                // The result of a pointer add or sub is an alias to the pointer.
                                // Add uses of this instruction to worklist.
                                def_use
                                    .get(&value)
                                    .cloned()
                                    .unwrap_or_default()
                                    .iter()
                                    .for_each(|r#use| {
                                        in_worklist.insert(*r#use);
                                        worklist.push(*r#use);
                                    });
                            }
                            BinaryOpKind::Mul
                            | BinaryOpKind::Div
                            | BinaryOpKind::And
                            | BinaryOpKind::Or
                            | BinaryOpKind::Xor
                            | BinaryOpKind::Mod
                            | BinaryOpKind::Rsh
                            | BinaryOpKind::Lsh => {
                                panic!("Pointers shouldn't be used in these operations");
                            }
                        }
                    }
                    InstOp::PtrToInt(..)
                    | InstOp::ContractCall { .. }
                    | InstOp::AsmBlock(..)
                    | InstOp::Store { .. } => {
                        // It's a store, or we can't trace this anymore. Assume the worst.
                        *arg_mutabilities.get_mut(arg_idx).unwrap() = ArgPointeeMutability::Mutable;
                        continue 'analyse_next_arg;
                    }
                    InstOp::CastPtr(..) | InstOp::GetElemPtr { .. } => {
                        // The result is now an alias of the argument. Process it.
                        def_use
                            .get(&value)
                            .cloned()
                            .unwrap_or_default()
                            .iter()
                            .for_each(|r#use| {
                                in_worklist.insert(*r#use);
                                worklist.push(*r#use);
                            });
                    }
                    InstOp::Load(_) => {
                        // Since we don't worry about pointers that are indirectly mutated,
                        // (i.e., inside the loaded value) we're done here.
                    }
                    InstOp::MemClearVal { dst_val_ptr }
                    | InstOp::MemCopyBytes { dst_val_ptr, .. }
                    | InstOp::MemCopyVal { dst_val_ptr, .. } => {
                        // If the destination is an alias of the argument pointer,
                        // then the argument is being mutated. (We could be here
                        // because the source pointer is a use of the argument pointer,
                        // but that doesn't indicate mutability).
                        if in_worklist.contains(dst_val_ptr) || aliases.contains(dst_val_ptr) {
                            // If the destination pointer is the same as the argument pointer,
                            // we can assume that the pointee is mutable.
                            *arg_mutabilities.get_mut(arg_idx).unwrap() =
                                ArgPointeeMutability::Mutable;
                            continue 'analyse_next_arg;
                        }
                    }
                    InstOp::Call(callee, actual_params) => {
                        let Some(callee_mutability) = res.0.get(callee) else {
                            // assume the worst.x
                            *arg_mutabilities.get_mut(arg_idx).unwrap() =
                                ArgPointeeMutability::Mutable;
                            continue 'analyse_next_arg;
                        };
                        for (caller_param_idx, caller_param) in actual_params.iter().enumerate() {
                            if callee_mutability[caller_param_idx] == ArgPointeeMutability::Mutable
                            {
                                // The callee mutates the parameter at caller_param_idx
                                // If what we're passing at that position is an alias of our argument,
                                // then we mark that our argument is mutable.
                                if in_worklist.contains(caller_param)
                                    || aliases.contains(caller_param)
                                {
                                    *arg_mutabilities.get_mut(arg_idx).unwrap() =
                                        ArgPointeeMutability::Mutable;
                                }
                            }
                        }
                    }
                    InstOp::FuelVm(vmop) => match vmop {
                        FuelVmInstruction::Gtf { .. }
                        | FuelVmInstruction::Log { .. }
                        | FuelVmInstruction::ReadRegister(_)
                        | FuelVmInstruction::Revert(_)
                        | FuelVmInstruction::JmpMem
                        | FuelVmInstruction::Smo { .. }
                        | FuelVmInstruction::StateClear { .. } => {}
                        FuelVmInstruction::StateLoadQuadWord { load_val, .. } => {
                            // If the loaded value is an alias of the argument pointer,
                            // then the argument is being mutated.
                            if in_worklist.contains(load_val) || aliases.contains(load_val) {
                                *arg_mutabilities.get_mut(arg_idx).unwrap() =
                                    ArgPointeeMutability::Mutable;
                                continue 'analyse_next_arg;
                            }
                        }
                        FuelVmInstruction::StateLoadWord(_)
                        | FuelVmInstruction::StateStoreWord { .. } => {}
                        FuelVmInstruction::StateStoreQuadWord { .. } => {}
                        FuelVmInstruction::WideUnaryOp { result, .. }
                        | FuelVmInstruction::WideBinaryOp { result, .. }
                        | FuelVmInstruction::WideModularOp { result, .. } => {
                            // If the result is an alias of the argument pointer,
                            // then the argument is being mutated.
                            if in_worklist.contains(result) || aliases.contains(result) {
                                *arg_mutabilities.get_mut(arg_idx).unwrap() =
                                    ArgPointeeMutability::Mutable;
                                continue 'analyse_next_arg;
                            }
                        }
                        FuelVmInstruction::WideCmpOp { .. } => {}
                        FuelVmInstruction::Retd { .. } => {}
                    },
                },
                ValueDatum::Argument(_) => {
                    // Add all users of this argument to the worklist.
                    def_use
                        .get(&value)
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .for_each(|r#use| {
                            in_worklist.insert(*r#use);
                            worklist.push(*r#use);
                        });
                }
                ValueDatum::Constant(_) => panic!("Constants cannot be users"),
            }
        }
    }

    res.0.insert(function, arg_mutabilities);

    Ok(())
}
