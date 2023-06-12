//! Optimisations related to mem_copy.
//! - replace a `store` directly from a `load` with a `mem_copy_val`.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    combine_indices, get_symbol, get_symbols, may_alias, must_alias, pointee_size, AnalysisResults,
    Block, Context, EscapedSymbols, Function, Instruction, IrError, LocalVar, Pass, PassMutability,
    ScopedPass, Symbol, Type, Value, ValueDatum, ESCAPED_SYMBOLS_NAME,
};

pub const MEMCPYOPT_NAME: &str = "memcpyopt";

pub fn create_memcpyopt_pass() -> Pass {
    Pass {
        name: MEMCPYOPT_NAME,
        descr: "Memcopy optimization.",
        deps: vec![ESCAPED_SYMBOLS_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(mem_copy_opt)),
    }
}

pub fn mem_copy_opt(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    modified |= local_copy_prop_prememcpy(context, function)?;
    modified |= load_store_to_memcopy(context, function)?;
    modified |= local_copy_prop(context, analyses, function)?;

    Ok(modified)
}

struct InstInfo {
    // The block in which an instruction is
    block: Block,
    // Relative (use only for comparison) position of instruction in `block`.
    pos: usize,
}

fn local_copy_prop_prememcpy(context: &mut Context, function: Function) -> Result<bool, IrError> {
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

/// Copy propagation of `memcpy`s within a block.
fn local_copy_prop(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let escaped_symbols: &EscapedSymbols = analyses.get_analysis_result(function);

    // Currently (as we scan a block) available `memcpy`s.
    let mut available_copies: FxHashSet<Value>;
    // Map a symbol to the available `memcpy`s of which its a source.
    let mut src_to_copies: FxHashMap<Symbol, FxHashSet<Value>>;
    // Map a symbol to the available `memcpy`s of which its a destination.
    // (multiple memcpys for the same destination may be available when
    // they are partial / field writes, and don't alias).
    let mut dest_to_copies: FxHashMap<Symbol, FxHashSet<Value>>;

    // If a value (symbol) is found to be defined, remove it from our tracking.
    fn kill_defined_symbol(
        context: &Context,
        value: Value,
        len: u64,
        available_copies: &mut FxHashSet<Value>,
        src_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
        dest_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
    ) {
        let syms = get_symbols(context, value);
        for sym in syms {
            if let Some(copies) = src_to_copies.get_mut(&sym) {
                for copy in &*copies {
                    let (_, src_ptr, copy_size) = deconstruct_memcpy(context, *copy);
                    if may_alias(context, value, len, src_ptr, copy_size) {
                        available_copies.remove(copy);
                    }
                }
                copies.retain(|copy| available_copies.contains(copy));
            }
            if let Some(copies) = dest_to_copies.get_mut(&sym) {
                for copy in &*copies {
                    let (dest_ptr, copy_size) = match copy.get_instruction(context).unwrap() {
                        Instruction::MemCopyBytes {
                            dst_val_ptr,
                            src_val_ptr: _,
                            byte_len,
                        } => (*dst_val_ptr, *byte_len),
                        Instruction::MemCopyVal {
                            dst_val_ptr,
                            src_val_ptr: _,
                        } => (*dst_val_ptr, pointee_size(context, *dst_val_ptr)),
                        _ => panic!("Unexpected copy instruction"),
                    };
                    if may_alias(context, value, len, dest_ptr, copy_size) {
                        available_copies.remove(copy);
                    }
                }
                copies.retain(|copy| available_copies.contains(copy));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_new_copy(
        context: &Context,
        escaped_symbols: &EscapedSymbols,
        copy_inst: Value,
        dst_val_ptr: Value,
        src_val_ptr: Value,
        available_copies: &mut FxHashSet<Value>,
        src_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
        dest_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
    ) {
        if let (Some(dst_sym), Some(src_sym)) = (
            get_symbol(context, dst_val_ptr),
            get_symbol(context, src_val_ptr),
        ) {
            if escaped_symbols.contains(&dst_sym) || escaped_symbols.contains(&src_sym) {
                return;
            }
            dest_to_copies
                .entry(dst_sym)
                .and_modify(|set| {
                    set.insert(copy_inst);
                })
                .or_insert([copy_inst].into_iter().collect());
            src_to_copies
                .entry(src_sym)
                .and_modify(|set| {
                    set.insert(copy_inst);
                })
                .or_insert([copy_inst].into_iter().collect());
            available_copies.insert(copy_inst);
        }
    }

    // Deconstruct a memcpy into (dst_val_ptr, src_val_ptr, copy_len).
    fn deconstruct_memcpy(context: &Context, inst: Value) -> (Value, Value, u64) {
        match inst.get_instruction(context).unwrap() {
            Instruction::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len,
            } => (*dst_val_ptr, *src_val_ptr, *byte_len),
            Instruction::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => (
                *dst_val_ptr,
                *src_val_ptr,
                pointee_size(context, *dst_val_ptr),
            ),
            _ => unreachable!("Only memcpy instructions handled"),
        }
    }

    struct ReplGep {
        base: Symbol,
        elem_ptr_ty: Type,
        indices: Vec<Value>,
    }
    enum Replacement {
        OldGep(Value),
        NewGep(ReplGep),
    }

    fn process_load(
        context: &Context,
        escaped_symbols: &EscapedSymbols,
        inst: Value,
        src_val_ptr: Value,
        dest_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
        replacements: &mut FxHashMap<Value, Replacement>,
    ) -> bool {
        // For every `memcpy` that src_val_ptr is a destination of,
        // check if we can do the load from the source of that memcpy.
        if let Some(src_sym) = get_symbol(context, src_val_ptr) {
            if escaped_symbols.contains(&src_sym) {
                return false;
            }
            for memcpy in dest_to_copies
                .get(&src_sym)
                .iter()
                .flat_map(|set| set.iter())
            {
                let (dst_ptr_memcpy, src_ptr_memcpy, copy_len) =
                    deconstruct_memcpy(context, *memcpy);
                // If the location where we're loading from exactly matches the destination of
                // the memcpy, just load from the source pointer of the memcpy.
                // TODO: In both the arms below, we check that the pointer type
                // matches. This isn't really needed as the copy happens and the
                // data we want is safe to access. But we just don't know how to
                // generate the right GEP always. So that's left for another day.
                if must_alias(
                    context,
                    src_val_ptr,
                    pointee_size(context, src_val_ptr),
                    dst_ptr_memcpy,
                    copy_len,
                ) {
                    // Replace src_val_ptr with src_ptr_memcpy.
                    if src_val_ptr.get_type(context) == src_ptr_memcpy.get_type(context) {
                        replacements.insert(inst, Replacement::OldGep(src_ptr_memcpy));
                        return true;
                    }
                } else {
                    // if the memcpy copies the entire symbol, we could
                    // insert a new GEP from the source of the memcpy.
                    if let (Some(memcpy_src_sym), Some(memcpy_dst_sym), Some(new_indices)) = (
                        get_symbol(context, src_ptr_memcpy),
                        get_symbol(context, dst_ptr_memcpy),
                        combine_indices(context, src_val_ptr),
                    ) {
                        let memcpy_src_sym_type = memcpy_src_sym
                            .get_type(context)
                            .get_pointee_type(context)
                            .unwrap();
                        let memcpy_dst_sym_type = memcpy_dst_sym
                            .get_type(context)
                            .get_pointee_type(context)
                            .unwrap();
                        if memcpy_src_sym_type == memcpy_dst_sym_type
                            && memcpy_dst_sym_type.size_in_bytes(context) == copy_len
                        {
                            replacements.insert(
                                inst,
                                Replacement::NewGep(ReplGep {
                                    base: memcpy_src_sym,
                                    elem_ptr_ty: src_val_ptr.get_type(context).unwrap(),
                                    indices: new_indices,
                                }),
                            );
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    let mut modified = false;
    for block in function.block_iter(context) {
        // A `memcpy` itself has a `load`, so we can `process_load` on it.
        // If now, we've marked the source of this `memcpy` for optimization,
        // it itself cannot be "generated" as a new candidate `memcpy`.
        // This is the reason we run a loop on the block till there's no more
        // optimization possible. We could track just the changes and do it
        // all in one go, but that would complicate the algorithm. So I've
        // marked this as a TODO for now (#4600).
        loop {
            available_copies = FxHashSet::default();
            src_to_copies = FxHashMap::default();
            dest_to_copies = FxHashMap::default();

            // Replace the load/memcpy source pointer with something else.
            let mut replacements = FxHashMap::default();

            fn kill_escape_args(
                context: &Context,
                args: &Vec<Value>,
                available_copies: &mut FxHashSet<Value>,
                src_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
                dest_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
            ) {
                for arg in args {
                    let max_size = get_symbols(context, *arg)
                        .iter()
                        .filter_map(|sym| {
                            sym.get_type(context)
                                .get_pointee_type(context)
                                .map(|pt| pt.size_in_bytes(context))
                        })
                        .max()
                        .unwrap_or(0);
                    kill_defined_symbol(
                        context,
                        *arg,
                        max_size,
                        available_copies,
                        src_to_copies,
                        dest_to_copies,
                    );
                }
            }

            for inst in block.instruction_iter(context) {
                match inst.get_instruction(context).unwrap() {
                    Instruction::Call(_, args) => kill_escape_args(
                        context,
                        args,
                        &mut available_copies,
                        &mut src_to_copies,
                        &mut dest_to_copies,
                    ),
                    Instruction::AsmBlock(_, args) => {
                        let args = args.iter().filter_map(|arg| arg.initializer).collect();
                        kill_escape_args(
                            context,
                            &args,
                            &mut available_copies,
                            &mut src_to_copies,
                            &mut dest_to_copies,
                        );
                    }
                    Instruction::IntToPtr(_, _) => {
                        // The only safe thing we can do is to clear all information.
                        available_copies.clear();
                        src_to_copies.clear();
                        dest_to_copies.clear();
                    }
                    Instruction::Load(src_val_ptr) => {
                        process_load(
                            context,
                            escaped_symbols,
                            inst,
                            *src_val_ptr,
                            &mut dest_to_copies,
                            &mut replacements,
                        );
                    }
                    Instruction::MemCopyBytes { .. } | Instruction::MemCopyVal { .. } => {
                        let (dst_val_ptr, src_val_ptr, copy_len) =
                            deconstruct_memcpy(context, inst);
                        kill_defined_symbol(
                            context,
                            dst_val_ptr,
                            copy_len,
                            &mut available_copies,
                            &mut src_to_copies,
                            &mut dest_to_copies,
                        );
                        // If this memcpy itself can be optimized, we do just that, and not "gen" a new one.
                        if !process_load(
                            context,
                            escaped_symbols,
                            inst,
                            src_val_ptr,
                            &mut dest_to_copies,
                            &mut replacements,
                        ) {
                            gen_new_copy(
                                context,
                                escaped_symbols,
                                inst,
                                dst_val_ptr,
                                src_val_ptr,
                                &mut available_copies,
                                &mut src_to_copies,
                                &mut dest_to_copies,
                            );
                        }
                    }
                    Instruction::Store {
                        dst_val_ptr,
                        stored_val: _,
                    } => {
                        kill_defined_symbol(
                            context,
                            *dst_val_ptr,
                            pointee_size(context, *dst_val_ptr),
                            &mut available_copies,
                            &mut src_to_copies,
                            &mut dest_to_copies,
                        );
                    }
                    _ => (),
                }
            }

            if replacements.is_empty() {
                break;
            } else {
                modified = true;
            }

            // If we have any NewGep replacements, insert those new GEPs into the block.
            // Since the new instructions need to be just before the value load that they're
            // going to be used in, we copy all the instructions into a new vec
            // and just replace the contents of the basic block.
            let mut new_insts = vec![];
            for inst in block.instruction_iter(context) {
                if let Some(replacement) = replacements.remove(&inst) {
                    let replacement = match replacement {
                        Replacement::OldGep(v) => v,
                        Replacement::NewGep(ReplGep {
                            base,
                            elem_ptr_ty,
                            indices,
                        }) => {
                            let base = match base {
                                Symbol::Local(local) => {
                                    let base = Value::new_instruction(
                                        context,
                                        Instruction::GetLocal(local),
                                    );
                                    new_insts.push(base);
                                    base
                                }
                                Symbol::Arg(block_arg) => {
                                    block_arg.block.get_arg(context, block_arg.idx).unwrap()
                                }
                            };
                            let v = Value::new_instruction(
                                context,
                                Instruction::GetElemPtr {
                                    base,
                                    elem_ptr_ty,
                                    indices,
                                },
                            );
                            new_insts.push(v);
                            v
                        }
                    };
                    match inst.get_instruction_mut(context) {
                        Some(Instruction::Load(ref mut src_val_ptr))
                        | Some(Instruction::MemCopyBytes {
                            ref mut src_val_ptr,
                            ..
                        })
                        | Some(Instruction::MemCopyVal {
                            ref mut src_val_ptr,
                            ..
                        }) => *src_val_ptr = replacement,
                        _ => panic!("Unexpected instruction type"),
                    }
                }
                new_insts.push(inst);
            }

            // Replace the basic block contents with what we just built.
            let _ = std::mem::replace(&mut (context.blocks[block.0].instructions), new_insts);
        }
    }

    Ok(modified)
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

    let src_symbols = get_symbols(context, src_ptr);

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
                if get_symbols(context, *dst_val_ptr)
                    .iter()
                    .any(|sym| src_symbols.contains(sym))
                {
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
