//! Optimisations related to mem_copy.
//! - replace a `store` directly from a `load` with a `mem_copy_val`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    AnalysisResults, Block, BlockArgument, Context, Function, Instruction, IrError, LocalVar, Pass,
    PassMutability, ScopedPass, Type, Value, ValueDatum,
};

pub const MEMCPYOPT_NAME: &str = "memcpyopt";

pub fn create_memcpyopt_pass() -> Pass {
    Pass {
        name: MEMCPYOPT_NAME,
        descr: "Memcopy optimization.",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(mem_copy_opt)),
    }
}

pub fn mem_copy_opt(
    context: &mut Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    modified |= local_copy_prop(context, function)?;
    modified |= load_store_to_memcopy(context, function)?;

    Ok(modified)
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
enum Symbol {
    Local(LocalVar),
    Arg(BlockArgument),
}

impl Symbol {
    pub fn get_type(&self, context: &Context) -> Type {
        match self {
            Symbol::Local(l) => l.get_type(context),
            Symbol::Arg(ba) => ba.ty,
        }
    }

    pub fn _get_name(&self, context: &Context, function: Function) -> String {
        match self {
            Symbol::Local(l) => function.lookup_local_name(context, l).unwrap().clone(),
            Symbol::Arg(ba) => format!("{}[{}]", ba.block.get_label(context), ba.idx),
        }
    }
}

fn get_symbol(context: &Context, val: Value) -> Option<Symbol> {
    match context.values[val.0].value {
        ValueDatum::Instruction(Instruction::GetLocal(local)) => Some(Symbol::Local(local)),
        ValueDatum::Instruction(Instruction::GetElemPtr { base, .. }) => get_symbol(context, base),
        ValueDatum::Argument(b) => Some(Symbol::Arg(b)),
        _ => None,
    }
}

struct InstInfo {
    // The block in which an instruction is
    block: Block,
    // Relative (use only for comparison) position of instruction in `block`.
    pos: usize,
}

/// Copy propagation of loads+store (i.e., a memory copy) requires
/// a data-flow analysis. Until then, we do a safe approximation,
/// restricting to when every related instruction is in the same block.
fn local_copy_prop(context: &mut Context, function: Function) -> Result<bool, IrError> {
    let mut loads_map = FxHashMap::<Symbol, Vec<Value>>::default();
    let mut stores_map = FxHashMap::<Symbol, Vec<Value>>::default();
    let mut instr_info_map = FxHashMap::<Value, InstInfo>::default();
    let mut asm_uses = FxHashSet::<Symbol>::default();

    for (pos, (block, inst)) in function.instruction_iter(context).enumerate() {
        let info = || InstInfo { block, pos };
        let inst_e = inst.get_instruction(context).unwrap();
        match inst_e {
            Instruction::Load(src_val_ptr) => {
                if let Some(local) = get_symbol(context, *src_val_ptr) {
                    loads_map
                        .entry(local)
                        .and_modify(|loads| loads.push(inst))
                        .or_insert(vec![inst]);
                    instr_info_map.insert(inst, info());
                }
            }
            Instruction::Store { dst_val_ptr, .. } => {
                if let Some(local) = get_symbol(context, *dst_val_ptr) {
                    stores_map
                        .entry(local)
                        .and_modify(|stores| stores.push(inst))
                        .or_insert(vec![inst]);
                    instr_info_map.insert(inst, info());
                }
            }
            Instruction::AsmBlock(_, args) => {
                for arg in args {
                    if let Some(arg) = arg.initializer {
                        if let Some(local) = get_symbol(context, arg) {
                            asm_uses.insert(local);
                        }
                    }
                }
            }
            _ => (),
        }
    }

    let mut to_delete = FxHashSet::<Value>::default();
    let candidates: FxHashMap<Symbol, Symbol> = function
        .instruction_iter(context)
        .enumerate()
        .filter_map(|(pos, (block, instr_val))| {
            instr_val
                .get_instruction(context)
                .and_then(|instr| {
                    // Is the instruction a Store?
                    if let Instruction::Store {
                        dst_val_ptr,
                        stored_val,
                    } = instr
                    {
                        get_symbol(context, *dst_val_ptr).and_then(|dst_local| {
                            stored_val
                                .get_instruction(context)
                                .map(|src_instr| (src_instr, stored_val, dst_local))
                        })
                    } else {
                        None
                    }
                })
                .and_then(|(src_instr, stored_val, dst_local)| {
                    // Is the Store source a Load?
                    if let Instruction::Load(src_val_ptr) = src_instr {
                        get_symbol(context, *src_val_ptr)
                            .map(|src_local| (stored_val, dst_local, src_local))
                    } else {
                        None
                    }
                })
                .and_then(|(src_load, dst_local, src_local)| {
                    let (temp_empty1, temp_empty2, temp_empty3) = (vec![], vec![], vec![]);
                    let dst_local_stores = stores_map.get(&dst_local).unwrap_or(&temp_empty1);
                    let src_local_stores = stores_map.get(&src_local).unwrap_or(&temp_empty2);
                    let dst_local_loads = loads_map.get(&dst_local).unwrap_or(&temp_empty3);
                    // This must be the only store of dst_local.
                    if dst_local_stores.len() != 1 || dst_local_stores[0] != instr_val
                        ||
                        // All stores of src_local must be in the same block, prior to src_load.
                        !src_local_stores.iter().all(|store_val|{
                            let instr_info = instr_info_map.get(store_val).unwrap();
                            let src_load_info = instr_info_map.get(src_load).unwrap();
                            instr_info.block == block && instr_info.pos < src_load_info.pos
                        })
                        ||
                        // All loads of dst_local must be after this instruction, in the same block.
                        !dst_local_loads.iter().all(|load_val| {
                            let instr_info = instr_info_map.get(load_val).unwrap();
                            instr_info.block == block && instr_info.pos > pos
                        })
                        // We don't deal with ASM blocks.
                        || asm_uses.contains(&dst_local)
                        // We don't deal part copies.
                        || dst_local.get_type(context) != src_local.get_type(context)
                        // We don't replace the destination when it's an arg.
                        || matches!(dst_local, Symbol::Arg(_))
                    {
                        None
                    } else {
                        to_delete.insert(instr_val);
                        Some((dst_local, src_local))
                    }
                })
        })
        .collect();

    // if we have A replaces B and B replaces C, then A must replace C also.
    fn closure(candidates: &FxHashMap<Symbol, Symbol>, src_local: &Symbol) -> Option<Symbol> {
        candidates
            .get(src_local)
            .map(|replace_with| closure(candidates, replace_with).unwrap_or(*replace_with))
    }

    // If the source is an Arg, we replace uses of destination with Arg.
    // otherwise (`get_local`), we replace the local symbol in-place.
    enum ReplaceWith {
        InPlaceLocal(LocalVar),
        Value(Value),
    }

    // Because we can't borrow context for both iterating and replacing, do it in 2 steps.
    let replaces: Vec<_> = function
        .instruction_iter(context)
        .filter_map(|(_block, value)| match value.get_instruction(context) {
            Some(Instruction::GetLocal(local)) => {
                closure(&candidates, &Symbol::Local(*local)).map(|replace_with| {
                    (
                        value,
                        match replace_with {
                            Symbol::Local(local) => ReplaceWith::InPlaceLocal(local),
                            Symbol::Arg(ba) => {
                                ReplaceWith::Value(ba.block.get_arg(context, ba.idx).unwrap())
                            }
                        },
                    )
                })
            }
            _ => None,
        })
        .collect();

    let mut value_replace = FxHashMap::<Value, Value>::default();
    for (value, replace_with) in replaces.into_iter() {
        match replace_with {
            ReplaceWith::InPlaceLocal(replacement_var) => {
                let Some(Instruction::GetLocal(redundant_var)) = value.get_instruction(context) else {
                    panic!("earlier match now fails");
                };
                if redundant_var.is_mutable(context) {
                    replacement_var.set_mutable(context, true);
                }
                value.replace(
                    context,
                    ValueDatum::Instruction(Instruction::GetLocal(replacement_var)),
                )
            }
            ReplaceWith::Value(replace_with) => {
                value_replace.insert(value, replace_with);
            }
        }
    }
    function.replace_values(context, &value_replace, None);

    // Delete stores to the replaced local.
    let blocks: Vec<Block> = function.block_iter(context).collect();
    for block in blocks {
        block.remove_instructions(context, |value| to_delete.contains(&value));
    }
    Ok(true)
}

// Is (an alias of) src_ptr clobbered on any path from load_val to store_val?
fn is_clobbered(
    context: &Context,
    store_block: Block,
    store_val: Value,
    load_val: Value,
    src_ptr: Value,
) -> bool {
    let mut iter = store_block
        .instruction_iter(context)
        .rev()
        .skip_while(|i| i != &store_val);
    assert!(iter.next().unwrap() == store_val);

    // Scan backwards till we encounter load_val, checking if
    // any store aliases with src_ptr.
    let mut worklist: Vec<(Block, Box<dyn Iterator<Item = Value>>)> =
        vec![(store_block, Box::new(iter))];
    let mut visited = FxHashSet::default();
    'next_job: while !worklist.is_empty() {
        let (block, iter) = worklist.pop().unwrap();
        visited.insert(block);
        for inst in iter {
            if inst == load_val || inst == store_val {
                // We don't need to go beyond either the source load or the candidate store.
                continue 'next_job;
            }
            if let Some(Instruction::Store {
                dst_val_ptr,
                stored_val: _,
            }) = inst.get_instruction(context)
            {
                if get_symbol(context, *dst_val_ptr) == get_symbol(context, src_ptr) {
                    return true;
                }
            }
        }
        for pred in block.pred_iter(context) {
            if !visited.contains(pred) {
                worklist.push((
                    *pred,
                    Box::new(pred.instruction_iter(context).rev().skip_while(|_| false)),
                ));
            }
        }
    }

    false
}

fn load_store_to_memcopy(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find any `store`s of `load`s.  These can be replaced with `mem_copy` and are especially
    // important for non-copy types on architectures which don't support loading them.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, store_instr_val)| {
            store_instr_val
                .get_instruction(context)
                .and_then(|instr| {
                    // Is the instruction a Store?
                    if let Instruction::Store {
                        dst_val_ptr,
                        stored_val,
                    } = instr
                    {
                        stored_val
                            .get_instruction(context)
                            .map(|src_instr| (*stored_val, src_instr, dst_val_ptr))
                    } else {
                        None
                    }
                })
                .and_then(|(src_instr_val, src_instr, dst_val_ptr)| {
                    // Is the Store source a Load?
                    if let Instruction::Load(src_val_ptr) = src_instr {
                        Some((
                            block,
                            src_instr_val,
                            store_instr_val,
                            *dst_val_ptr,
                            *src_val_ptr,
                        ))
                    } else {
                        None
                    }
                })
                .and_then(
                    |candidate @ (block, load_val, store_val, _dst_ptr, src_ptr)| {
                        // Ensure that there's no path from load_val to store_val that might overwrite src_ptr.
                        (!is_clobbered(context, block, store_val, load_val, src_ptr))
                            .then_some(candidate)
                    },
                )
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    for (block, _src_instr_val, store_val, dst_val_ptr, src_val_ptr) in candidates {
        let mem_copy_val = Value::new_instruction(
            context,
            Instruction::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            },
        );
        block.replace_instruction(context, store_val, mem_copy_val)?;
    }

    Ok(true)
}
