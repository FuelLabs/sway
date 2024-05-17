//! ## Dead Code Elimination
//!
//! This optimization removes unused definitions. The pass is a combination of
//!   1. A liveness analysis that keeps track of the uses of a definition,
//!   2. At the time of inspecting a definition, if it has no uses, it is removed.
//! This pass does not do CFG transformations. That is handled by simplify_cfg.

use rustc_hash::FxHashSet;

use crate::{
    get_gep_referred_symbols, get_referred_symbols, memory_utils, AnalysisResults, Context,
    EscapedSymbols, Function, InstOp, Instruction, IrError, LocalVar, Module, Pass, PassMutability,
    ReferredSymbols, ScopedPass, Symbol, Value, ValueDatum, ESCAPED_SYMBOLS_NAME,
};

use std::collections::{HashMap, HashSet};

pub const DCE_NAME: &str = "dce";

pub fn create_dce_pass() -> Pass {
    Pass {
        name: DCE_NAME,
        descr: "Dead code elimination",
        runner: ScopedPass::FunctionPass(PassMutability::Transform(dce)),
        deps: vec![ESCAPED_SYMBOLS_NAME],
    }
}

pub const FN_DCE_NAME: &str = "fn-dce";

pub fn create_fn_dce_pass() -> Pass {
    Pass {
        name: FN_DCE_NAME,
        descr: "Dead function elimination",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(fn_dce)),
    }
}

fn can_eliminate_instruction(
    context: &Context,
    val: Value,
    num_symbol_uses: &HashMap<Symbol, u32>,
    escaped_symbols: &EscapedSymbols,
) -> bool {
    let inst = val.get_instruction(context).unwrap();
    (!inst.op.is_terminator() && !inst.op.may_have_side_effect())
        || is_removable_store(context, val, num_symbol_uses, escaped_symbols)
}

fn is_removable_store(
    context: &Context,
    val: Value,
    num_symbol_uses: &HashMap<Symbol, u32>,
    escaped_symbols: &EscapedSymbols,
) -> bool {
    match val.get_instruction(context).unwrap().op {
        InstOp::MemCopyBytes { dst_val_ptr, .. }
        | InstOp::MemCopyVal { dst_val_ptr, .. }
        | InstOp::Store { dst_val_ptr, .. } => {
            let syms = get_referred_symbols(context, dst_val_ptr);
            match syms {
                ReferredSymbols::Complete(syms) => syms.iter().all(|sym| {
                    !escaped_symbols.contains(sym)
                        && num_symbol_uses.get(sym).map_or(0, |uses| *uses) == 0
                }),
                // We cannot guarantee that the destination is not used.
                ReferredSymbols::Incomplete(_) => false,
            }
        }
        _ => false,
    }
}

/// Perform dead code (if any) elimination and return true if function is modified.
pub fn dce(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let escaped_symbols: &EscapedSymbols = analyses.get_analysis_result(function);

    // Number of uses that an instruction has.
    let mut num_uses: HashMap<Value, u32> = HashMap::new();
    let mut num_local_uses: HashMap<LocalVar, u32> = HashMap::new();
    let mut num_symbol_uses: HashMap<Symbol, u32> = HashMap::new();
    let mut stores_of_sym: HashMap<Symbol, Vec<Value>> = HashMap::new();

    // Every argument is assumed to be loaded from (from the caller),
    // so stores to it shouldn't be eliminated.
    for sym in function
        .args_iter(context)
        .flat_map(|arg| get_gep_referred_symbols(context, arg.1))
    {
        num_symbol_uses
            .entry(sym)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    // Go through each instruction and update use_count.
    for (_block, inst) in function.instruction_iter(context) {
        for sym in memory_utils::get_loaded_symbols(context, inst) {
            num_symbol_uses
                .entry(sym)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        for stored_sym in memory_utils::get_stored_symbols(context, inst) {
            stores_of_sym
                .entry(stored_sym)
                .and_modify(|stores| stores.push(inst))
                .or_insert(vec![inst]);
        }

        let inst = inst.get_instruction(context).unwrap();
        if let InstOp::GetLocal(local) = inst.op {
            num_local_uses
                .entry(local)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        let opds = inst.op.get_operands();
        for v in opds {
            match context.values[v.0].value {
                ValueDatum::Instruction(_) => {
                    num_uses
                        .entry(v)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                ValueDatum::Configurable(_) | ValueDatum::Constant(_) | ValueDatum::Argument(_) => {
                }
            }
        }
    }

    let mut worklist = function
        .instruction_iter(context)
        .filter_map(|(_block, inst)| {
            (num_uses.get(&inst).is_none()
                || is_removable_store(context, inst, &num_symbol_uses, escaped_symbols))
            .then_some(inst)
        })
        .collect::<Vec<_>>();

    let mut modified = false;
    let mut cemetery = FxHashSet::default();
    while let Some(dead) = worklist.pop() {
        if !can_eliminate_instruction(context, dead, &num_symbol_uses, escaped_symbols)
            || cemetery.contains(&dead)
        {
            continue;
        }
        // Process dead's operands.
        let opds = dead.get_instruction(context).unwrap().op.get_operands();
        for v in opds {
            // Reduce the use count of v. If it reaches 0, add it to the worklist.
            match context.values[v.0].value {
                ValueDatum::Instruction(_) => {
                    let nu = num_uses.get_mut(&v).unwrap();
                    *nu -= 1;
                    if *nu == 0 {
                        worklist.push(v);
                    }
                }
                ValueDatum::Configurable(_) | ValueDatum::Constant(_) | ValueDatum::Argument(_) => {
                }
            }
        }
        for sym in memory_utils::get_loaded_symbols(context, dead) {
            let nu = num_symbol_uses.get_mut(&sym).unwrap();
            *nu -= 1;
            if *nu == 0 {
                for store in stores_of_sym.get(&sym).unwrap_or(&vec![]) {
                    worklist.push(*store);
                }
            }
        }
        cemetery.insert(dead);

        if let ValueDatum::Instruction(Instruction {
            op: InstOp::GetLocal(local),
            ..
        }) = context.values[dead.0].value
        {
            let count = num_local_uses.get_mut(&local).unwrap();
            *count -= 1;
        }
        modified = true;
    }

    // Remove all dead instructions.
    for block in function.block_iter(context) {
        block.remove_instructions(context, |inst| cemetery.contains(&inst));
    }

    let local_removals: Vec<_> = function
        .locals_iter(context)
        .filter_map(|(name, local)| {
            (num_local_uses.get(local).cloned().unwrap_or(0) == 0).then_some(name.clone())
        })
        .collect();
    if !local_removals.is_empty() {
        modified = true;
        function.remove_locals(context, &local_removals);
    }

    Ok(modified)
}

/// Remove entire functions from a module based on whether they are called or not, using a list of
/// root 'entry' functions to perform a search.
///
/// Functions which are `pub` will not be removed and only functions within the passed [`Module`]
/// are considered for removal.
pub fn fn_dce(context: &mut Context, _: &AnalysisResults, module: Module) -> Result<bool, IrError> {
    let entry_fns = module
        .function_iter(context)
        .filter(|func| func.is_entry(context) || func.is_fallback(context))
        .collect::<Vec<_>>();
    // Recursively find all the functions called by an entry function.
    fn grow_called_function_set(
        context: &Context,
        caller: Function,
        called_set: &mut HashSet<Function>,
    ) {
        if called_set.insert(caller) {
            // We haven't seen caller before.  Iterate for all that it calls.
            for func in caller
                .instruction_iter(context)
                .filter_map(|(_block, ins_value)| {
                    ins_value
                        .get_instruction(context)
                        .and_then(|ins| match &ins.op {
                            InstOp::Call(f, _args) => Some(f),
                            _otherwise => None,
                        })
                })
            {
                grow_called_function_set(context, *func, called_set);
            }
        }
    }

    // Gather our entry functions together into a set.
    let mut called_fns: HashSet<Function> = HashSet::new();
    for entry_fn in entry_fns {
        grow_called_function_set(context, entry_fn, &mut called_fns);
    }

    // Gather the functions in the module which aren't called.  It's better to collect them
    // separately first so as to avoid any issues with invalidating the function iterator.
    let dead_fns = module
        .function_iter(context)
        .filter(|f| !called_fns.contains(f))
        .collect::<Vec<_>>();

    let modified = !dead_fns.is_empty();
    for dead_fn in dead_fns {
        module.remove_function(context, &dead_fn);
    }

    Ok(modified)
}
