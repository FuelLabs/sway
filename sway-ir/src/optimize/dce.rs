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
    num_symbol_loaded: &NumSymbolLoaded,
    escaped_symbols: &EscapedSymbols,
) -> bool {
    let inst = val.get_instruction(context).unwrap();
    (!inst.op.is_terminator() && !inst.op.may_have_side_effect())
        || is_removable_store(context, val, num_symbol_loaded, escaped_symbols)
}

fn is_removable_store(
    context: &Context,
    val: Value,
    num_symbol_loaded: &NumSymbolLoaded,
    escaped_symbols: &EscapedSymbols,
) -> bool {
    let escaped_symbols = match escaped_symbols {
        EscapedSymbols::Complete(syms) => syms,
        EscapedSymbols::Incomplete(_) => return false,
    };

    let num_symbol_loaded = match num_symbol_loaded {
        NumSymbolLoaded::Unknown => return false,
        NumSymbolLoaded::Known(known_num_symbol_loaded) => known_num_symbol_loaded,
    };

    match val.get_instruction(context).unwrap().op {
        InstOp::MemCopyBytes { dst_val_ptr, .. }
        | InstOp::MemCopyVal { dst_val_ptr, .. }
        | InstOp::Store { dst_val_ptr, .. } => {
            let syms = get_referred_symbols(context, dst_val_ptr);
            match syms {
                ReferredSymbols::Complete(syms) => syms.iter().all(|sym| {
                    !escaped_symbols.contains(sym)
                        && num_symbol_loaded.get(sym).map_or(0, |uses| *uses) == 0
                }),
                // We cannot guarantee that the destination is not used.
                ReferredSymbols::Incomplete(_) => false,
            }
        }
        _ => false,
    }
}

/// How many times a [Symbol] gets loaded from, directly or indirectly.
/// This number is either exactly `Known` for all the symbols loaded from, or is
/// considered to be `Unknown` for all the symbols.
enum NumSymbolLoaded {
    Unknown,
    Known(HashMap<Symbol, u32>),
}

/// Instructions that store to a [Symbol], directly or indirectly.
/// These instructions are either exactly `Known` for all the symbols stored to, or is
/// considered to be `Unknown` for all the symbols.
enum StoresOfSymbol {
    Unknown,
    Known(HashMap<Symbol, Vec<Value>>),
}

/// Perform dead code (if any) elimination and return true if the `function` is modified.
pub fn dce(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // For DCE, we need to proceed with the analysis even if we have
    // incomplete list of escaped symbols, because we could have
    // unused instructions in code. Removing unused instructions is
    // independent of having any escaping symbols.
    let escaped_symbols: &EscapedSymbols = analyses.get_analysis_result(function);

    // Number of uses that an instruction has. This number is always known.
    let mut num_inst_uses: HashMap<Value, u32> = HashMap::new();
    // Number of times a local is accessed via `get_local`. This number is always known.
    let mut num_local_uses: HashMap<LocalVar, u32> = HashMap::new();
    // Number of times a symbol, local or a function argument, is loaded, directly or indirectly. This number can be unknown.
    let mut num_symbol_loaded: NumSymbolLoaded = NumSymbolLoaded::Known(HashMap::new());
    // Instructions that store to a symbol, directly or indirectly. This information can be unknown.
    let mut stores_of_sym: StoresOfSymbol = StoresOfSymbol::Known(HashMap::new());

    // TODO-IG: Update this logic once `mut arg: T`s are implemented.
    //          Currently, only `ref mut arg` arguments can be stored to,
    //          which means they can be loaded from the caller.
    //          Once we support `mut arg` in general, this will not be
    //          the case anymore and we will need to distinguish between
    //          `mut arg: T`, `arg: &mut T`, etc.
    // Every argument is assumed to be loaded from (from the caller),
    // so stores to it shouldn't be eliminated.
    if let NumSymbolLoaded::Known(known_num_symbol_loaded) = &mut num_symbol_loaded {
        for sym in function
            .args_iter(context)
            .flat_map(|arg| get_gep_referred_symbols(context, arg.1))
        {
            known_num_symbol_loaded
                .entry(sym)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    // Go through each instruction and update use counters.
    for (_block, inst) in function.instruction_iter(context) {
        if let NumSymbolLoaded::Known(known_num_symbol_loaded) = &mut num_symbol_loaded {
            match memory_utils::get_loaded_symbols(context, inst) {
                ReferredSymbols::Complete(loaded_symbols) => {
                    for sym in loaded_symbols {
                        known_num_symbol_loaded
                            .entry(sym)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }
                }
                ReferredSymbols::Incomplete(_) => num_symbol_loaded = NumSymbolLoaded::Unknown,
            }
        }

        if let StoresOfSymbol::Known(known_stores_of_sym) = &mut stores_of_sym {
            match memory_utils::get_stored_symbols(context, inst) {
                ReferredSymbols::Complete(stored_symbols) => {
                    for stored_sym in stored_symbols {
                        known_stores_of_sym
                            .entry(stored_sym)
                            .and_modify(|stores| stores.push(inst))
                            .or_insert(vec![inst]);
                    }
                }
                ReferredSymbols::Incomplete(_) => stores_of_sym = StoresOfSymbol::Unknown,
            }
        }

        // A local is used if it is accessed via `get_local`.
        let inst = inst.get_instruction(context).unwrap();
        if let InstOp::GetLocal(local) = inst.op {
            num_local_uses
                .entry(local)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        // An instruction is used if it is an operand in another instruction.
        let opds = inst.op.get_operands();
        for opd in opds {
            match context.values[opd.0].value {
                ValueDatum::Instruction(_) => {
                    num_inst_uses
                        .entry(opd)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                ValueDatum::Constant(_) | ValueDatum::Argument(_) => {}
            }
        }
    }

    // The list of all unused or `Store` instruction. Note that the `Store` instruction does
    // not result in a value, and will, thus, always be treated as unused and will not
    // have an entry in `num_inst_uses`. So, to collect unused or `Store` instructions it
    // is sufficient to filter those that are not used.
    let mut worklist = function
        .instruction_iter(context)
        .filter_map(|(_, inst)| num_inst_uses.get(&inst).is_none().then_some(inst))
        .collect::<Vec<_>>();

    let mut modified = false;
    let mut cemetery = FxHashSet::default();
    while let Some(dead) = worklist.pop() {
        if !can_eliminate_instruction(context, dead, &num_symbol_loaded, escaped_symbols)
            || cemetery.contains(&dead)
        {
            continue;
        }
        // Process dead's operands.
        let opds = dead.get_instruction(context).unwrap().op.get_operands();
        for opd in opds {
            // Reduce the use count of the operand used in the dead instruction.
            // If it reaches 0, add it to the worklist, since it is not used
            // anywhere else.
            match context.values[opd.0].value {
                ValueDatum::Instruction(_) => {
                    let nu = num_inst_uses.get_mut(&opd).unwrap();
                    *nu -= 1;
                    if *nu == 0 {
                        worklist.push(opd);
                    }
                }
                ValueDatum::Constant(_) | ValueDatum::Argument(_) => {}
            }
        }

        // If the `dead` instruction was the only instruction loading from a `sym`bol,
        // after removing it, there will be no loads anymore, so all the stores to
        // that `sym`bol can be added to the worklist.
        if let ReferredSymbols::Complete(loaded_symbols) =
            memory_utils::get_loaded_symbols(context, dead)
        {
            if let (
                NumSymbolLoaded::Known(known_num_symbol_loaded),
                StoresOfSymbol::Known(known_stores_of_sym),
            ) = (&mut num_symbol_loaded, &mut stores_of_sym)
            {
                for sym in loaded_symbols {
                    let nu = known_num_symbol_loaded.get_mut(&sym).unwrap();
                    *nu -= 1;
                    if *nu == 0 {
                        for store in known_stores_of_sym.get(&sym).unwrap_or(&vec![]) {
                            worklist.push(*store);
                        }
                    }
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
    let mut called_fns: HashSet<Function> = HashSet::new();

    // config decode fns
    for config in context.modules[module.0].configs.iter() {
        if let crate::ConfigContent::V1 { decode_fn, .. } = config.1 {
            grow_called_function_set(context, *decode_fn, &mut called_fns);
        }
    }

    // entry fns and fallback
    let entry_fns = module
        .function_iter(context)
        .filter(|func| func.is_entry(context) || func.is_fallback(context))
        .collect::<Vec<_>>();

    // expand all called fns
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
