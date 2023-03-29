//! Optimisations related to mem_copy.
//! - replace a `store` directly from a `load` with a `mem_copy_val`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    AnalysisResults, Block, Context, Function, Instruction, IrError, LocalVar, Pass,
    PassMutability, ScopedPass, Value, ValueDatum,
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
    let mut loads_map = FxHashMap::<LocalVar, Vec<Value>>::default();
    let mut stores_map = FxHashMap::<LocalVar, Vec<Value>>::default();
    let mut instr_info_map = FxHashMap::<Value, InstInfo>::default();
    let mut asm_uses = FxHashSet::<LocalVar>::default();

    fn get_local(context: &Context, val: Value) -> Option<LocalVar> {
        match val.get_instruction(context) {
            Some(Instruction::GetLocal(local)) => Some(*local),
            Some(Instruction::GetElemPtr { base, .. }) => get_local(context, *base),
            _ => None,
        }
    }

    for (pos, (block, inst)) in function.instruction_iter(context).enumerate() {
        let info = || InstInfo { block, pos };
        let inst_e = inst.get_instruction(context).unwrap();
        match inst_e {
            Instruction::Load(src_val_ptr) => {
                if let Some(local) = get_local(context, *src_val_ptr) {
                    loads_map
                        .entry(local)
                        .and_modify(|loads| loads.push(inst))
                        .or_insert(vec![inst]);
                    instr_info_map.insert(inst, info());
                }
            }
            Instruction::Store { dst_val_ptr, .. } => {
                if let Some(local) = get_local(context, *dst_val_ptr) {
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
                        if let Some(local) = get_local(context, arg) {
                            asm_uses.insert(local);
                        }
                    }
                }
            }
            _ => (),
        }
    }

    let mut to_delete = FxHashSet::<Value>::default();
    let candidates: FxHashMap<LocalVar, LocalVar> = function
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
                        get_local(context, *dst_val_ptr).and_then(|dst_local| {
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
                        get_local(context, *src_val_ptr)
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
    fn closure(
        candidates: &FxHashMap<LocalVar, LocalVar>,
        src_local: &LocalVar,
    ) -> Option<LocalVar> {
        candidates
            .get(src_local)
            .map(|replace_with| closure(candidates, replace_with).unwrap_or(*replace_with))
    }
    // Because we can't borrow context for both iterating and replacing, do it in 2 steps.
    let replaces: Vec<_> = function
        .instruction_iter(context)
        .filter_map(|(_block, value)| match value.get_instruction(context) {
            Some(Instruction::GetLocal(local)) => closure(&candidates, local).map(|replace_with| {
                (
                    value,
                    ValueDatum::Instruction(Instruction::GetLocal(replace_with)),
                )
            }),
            _ => None,
        })
        .collect();
    for (value, replace_with) in replaces.into_iter() {
        value.replace(context, replace_with);
    }

    // Delete stores to the replaced local.
    let blocks: Vec<Block> = function.block_iter(context).collect();
    for block in blocks {
        block.remove_instructions(context, |value| to_delete.contains(&value));
    }
    Ok(true)
}

fn load_store_to_memcopy(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find any `store`s of `load`s.  These can be replaced with `mem_copy` and are especially
    // important for non-copy types on architectures which don't support loading them.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val
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
                            .map(|src_instr| (src_instr, dst_val_ptr))
                    } else {
                        None
                    }
                })
                .and_then(|(src_instr, dst_val_ptr)| {
                    // Is the Store source a Load?
                    if let Instruction::Load(src_val_ptr) = src_instr {
                        Some((block, instr_val, *dst_val_ptr, *src_val_ptr))
                    } else {
                        None
                    }
                })
                .and_then(|candidate @ (_block, _store_val, dst_ptr, _src_ptr)| {
                    // XXX TEMPORARY 'FIX':
                    //
                    // We need to do proper aliasing analysis for this pass.  It's possible to have
                    // the following:
                    //
                    // X = load ptr A       -- dereference A
                    // store Y to ptr A     -- mutate A
                    // store X to ptr B     -- store original A to B
                    //
                    // Which this pass would convert to:
                    //
                    //                      -- DCE the load
                    // store Y to ptr A     -- mutate A
                    // memcpy ptr B, ptr A  -- copy _mutated_ A to B
                    //
                    // To temporarily avoid this problem we're not going to mem_copy copy types and
                    // assume (oh, no) that larger types subject to this pass aren't mutated.  This
                    // only works for now because it has always worked in the past, but there are
                    // no guarantees this couldn't flare up somewhere.
                    dst_ptr
                        .get_type(context)
                        .and_then(|ptr_ty| ptr_ty.get_pointee_type(context))
                        .map(|ty| !super::target_fuel::is_demotable_type(context, &ty))?
                        .then_some(candidate)
                })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    for (block, store_val, dst_val_ptr, src_val_ptr) in candidates {
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
