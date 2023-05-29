//! ## Dead Code Elimination
//!
//! This optimization removes unused definitions. The pass is a combination of
//!   1. A liveness analysis that keeps track of the uses of a definition,
//!   2. At the time of inspecting a definition, if it has no uses, it is removed.
//! This pass does not do CFG transformations. That is handled by simplify_cfg.

use rustc_hash::FxHashSet;

use crate::{
    get_symbols, AnalysisResults, Context, FuelVmInstruction, Function, Instruction, IrError,
    LocalVar, Module, Pass, PassMutability, ScopedPass, Symbol, Value, ValueDatum,
};

use std::collections::{HashMap, HashSet};

pub const DCE_NAME: &str = "dce";

pub fn create_dce_pass() -> Pass {
    Pass {
        name: DCE_NAME,
        descr: "Dead code elimination.",
        runner: ScopedPass::FunctionPass(PassMutability::Transform(dce)),
        deps: vec![],
    }
}

pub const FUNC_DCE_NAME: &str = "func_dce";

pub fn create_func_dce_pass() -> Pass {
    Pass {
        name: FUNC_DCE_NAME,
        descr: "Dead function elimination.",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(func_dce)),
    }
}

fn can_eliminate_instruction(
    context: &Context,
    val: Value,
    num_symbol_uses: &HashMap<Symbol, u32>,
) -> bool {
    let inst = val.get_instruction(context).unwrap();
    (!inst.is_terminator() && !inst.may_have_side_effect())
        || is_removable_store(context, val, num_symbol_uses)
}

fn get_loaded_symbols(context: &Context, val: Value) -> Vec<Symbol> {
    match val.get_instruction(context).unwrap() {
        Instruction::BinaryOp { .. }
        | Instruction::BitCast(_, _)
        | Instruction::Branch(_)
        | Instruction::ConditionalBranch { .. }
        | Instruction::Cmp(_, _, _)
        | Instruction::Nop
        | Instruction::CastPtr(_, _)
        | Instruction::GetLocal(_)
        | Instruction::GetElemPtr { .. }
        | Instruction::IntToPtr(_, _) => vec![],
        Instruction::PtrToInt(src_val_ptr, _) => {
            get_symbols(context, *src_val_ptr).iter().cloned().collect()
        }
        Instruction::ContractCall {
            params,
            coins,
            asset_id,
            ..
        } => vec![*params, *coins, *asset_id]
            .iter()
            .map(|val| {
                get_symbols(context, *val)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect(),
        Instruction::Call(_, args) => args
            .iter()
            .map(|val| {
                get_symbols(context, *val)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect(),
        Instruction::AsmBlock(_, args) => args
            .iter()
            .filter_map(|val| {
                val.initializer.and_then(|val| {
                    Some(
                        get_symbols(context, val)
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>(),
                    )
                })
            })
            .flatten()
            .collect(),
        Instruction::MemCopyBytes { src_val_ptr, .. }
        | Instruction::MemCopyVal { src_val_ptr, .. }
        | Instruction::Ret(src_val_ptr, _)
        | Instruction::Load(src_val_ptr)
        | Instruction::FuelVm(FuelVmInstruction::Smo {
            recipient_and_message: src_val_ptr,
            ..
        })
        | Instruction::FuelVm(FuelVmInstruction::StateLoadWord(src_val_ptr))
        | Instruction::FuelVm(FuelVmInstruction::StateStoreWord {
            key: src_val_ptr, ..
        })
        | Instruction::FuelVm(FuelVmInstruction::StateLoadQuadWord {
            key: src_val_ptr, ..
        })
        | Instruction::FuelVm(FuelVmInstruction::StateClear {
            key: src_val_ptr, ..
        }) => get_symbols(context, *src_val_ptr).iter().cloned().collect(),
        Instruction::FuelVm(FuelVmInstruction::StateStoreQuadWord {
            stored_val, key, ..
        }) => get_symbols(context, *stored_val)
            .iter()
            .cloned()
            .chain(get_symbols(context, *key).iter().cloned())
            .collect(),
        Instruction::Store { dst_val_ptr: _, .. } => vec![],
        Instruction::FuelVm(FuelVmInstruction::Gtf { .. })
        | Instruction::FuelVm(FuelVmInstruction::Log { .. })
        | Instruction::FuelVm(FuelVmInstruction::ReadRegister(_))
        | Instruction::FuelVm(FuelVmInstruction::Revert(_)) => vec![],
    }
}

fn get_stored_symbols(context: &Context, val: Value) -> Vec<Symbol> {
    match val.get_instruction(context).unwrap() {
        Instruction::BinaryOp { .. }
        | Instruction::BitCast(_, _)
        | Instruction::Branch(_)
        | Instruction::ConditionalBranch { .. }
        | Instruction::Cmp(_, _, _)
        | Instruction::Nop
        | Instruction::PtrToInt(_, _)
        | Instruction::Ret(_, _)
        | Instruction::CastPtr(_, _)
        | Instruction::GetLocal(_)
        | Instruction::GetElemPtr { .. }
        | Instruction::IntToPtr(_, _) => vec![],
        Instruction::ContractCall { params, .. } => get_symbols(context, *params),
        Instruction::Call(_, args) => args
            .iter()
            .map(|val| {
                get_symbols(context, *val)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect(),
        Instruction::AsmBlock(_, args) => args
            .iter()
            .filter_map(|val| {
                val.initializer.and_then(|val| {
                    Some(
                        get_symbols(context, val)
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>(),
                    )
                })
            })
            .flatten()
            .collect(),
        Instruction::MemCopyBytes { dst_val_ptr, .. }
        | Instruction::MemCopyVal { dst_val_ptr, .. }
        | Instruction::Store { dst_val_ptr, .. } => {
            get_symbols(context, *dst_val_ptr).iter().cloned().collect()
        }
        Instruction::Load(_) => vec![],
        Instruction::FuelVm(vmop) => match vmop {
            FuelVmInstruction::Gtf { .. }
            | FuelVmInstruction::Log { .. }
            | FuelVmInstruction::ReadRegister(_)
            | FuelVmInstruction::Revert(_)
            | FuelVmInstruction::Smo { .. }
            | FuelVmInstruction::StateClear { .. } => vec![],
            FuelVmInstruction::StateLoadQuadWord { load_val, .. } => {
                get_symbols(context, *load_val).iter().cloned().collect()
            }
            FuelVmInstruction::StateLoadWord(_) | FuelVmInstruction::StateStoreWord { .. } => {
                vec![]
            }
            FuelVmInstruction::StateStoreQuadWord { stored_val: _, .. } => vec![],
        },
    }
}

fn is_removable_store(
    context: &Context,
    val: Value,
    num_symbol_uses: &HashMap<Symbol, u32>,
) -> bool {
    match val.get_instruction(context).unwrap() {
        Instruction::MemCopyBytes { dst_val_ptr, .. }
        | Instruction::MemCopyVal { dst_val_ptr, .. }
        | Instruction::Store { dst_val_ptr, .. } => {
            let syms = get_symbols(context, *dst_val_ptr);
            syms.iter()
                .all(|sym| num_symbol_uses.get(&sym).map_or(0, |uses| *uses) == 0)
        }
        _ => false,
    }
}

/// Perform dead code (if any) elimination and return true if function modified.
pub fn dce(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // Number of uses that an instruction has.
    let mut num_uses: HashMap<Value, u32> = HashMap::new();
    let mut num_local_uses: HashMap<LocalVar, u32> = HashMap::new();
    let mut num_symbol_uses: HashMap<Symbol, u32> = HashMap::new();
    let mut stores_of_sym: HashMap<Symbol, Vec<Value>> = HashMap::new();

    // Go through each instruction and update use_count.
    for (_block, inst) in function.instruction_iter(context) {
        for sym in get_loaded_symbols(context, inst) {
            num_symbol_uses
                .entry(sym)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        for stored_sym in get_stored_symbols(context, inst) {
            stores_of_sym
                .entry(stored_sym)
                .and_modify(|stores| stores.push(inst))
                .or_insert(vec![inst]);
        }

        let inst = inst.get_instruction(context).unwrap();
        if let Instruction::GetLocal(local) = inst {
            num_local_uses
                .entry(*local)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        let opds = inst.get_operands();
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
            (num_uses.get(&inst).is_none() || is_removable_store(context, inst, &num_symbol_uses))
                .then_some(inst)
        })
        .collect::<Vec<_>>();

    let mut modified = false;
    let mut cemetery = FxHashSet::default();
    while !worklist.is_empty() {
        let dead = worklist.pop().unwrap();

        if !can_eliminate_instruction(context, dead, &num_symbol_uses) || cemetery.contains(&dead) {
            continue;
        }
        // Process dead's operands.
        let opds = dead.get_instruction(context).unwrap().get_operands();
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
        for sym in get_loaded_symbols(context, dead) {
            let nu = num_symbol_uses.get_mut(&sym).unwrap();
            *nu -= 1;
            if *nu == 0 {
                for store in stores_of_sym.get(&sym).unwrap_or(&vec![]) {
                    worklist.push(*store);
                }
            }
        }
        cemetery.insert(dead);

        if let ValueDatum::Instruction(Instruction::GetLocal(local)) = context.values[dead.0].value
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
pub fn func_dce(
    context: &mut Context,
    _: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    let entry_fns = module
        .function_iter(context)
        .filter(|func| func.is_entry(context))
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
                        .and_then(|ins| match ins {
                            Instruction::Call(f, _args) => Some(f),
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
