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
    modified |= load_store_to_memcopy(context, function)?;
    modified |= local_copy_prop(context, function)?;

    Ok(modified)
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub(crate) enum Symbol {
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

pub(crate) fn get_symbol(context: &Context, val: Value) -> Option<Symbol> {
    match context.values[val.0].value {
        ValueDatum::Instruction(Instruction::GetLocal(local)) => Some(Symbol::Local(local)),
        ValueDatum::Instruction(Instruction::GetElemPtr { base, .. }) => get_symbol(context, base),
        ValueDatum::Argument(b) => Some(Symbol::Arg(b)),
        _ => None,
    }
}

// Combine a series of GEPs into one.
fn combine_indices(context: &Context, val: Value) -> Option<Vec<Value>> {
    match &context.values[val.0].value {
        ValueDatum::Instruction(Instruction::GetLocal(_)) => Some(vec![]),
        ValueDatum::Instruction(Instruction::GetElemPtr {
            base,
            elem_ptr_ty: _,
            indices,
        }) => {
            let mut base_indices = combine_indices(context, *base)?;
            base_indices.append(&mut indices.clone());
            Some(base_indices)
        }
        ValueDatum::Argument(_) => Some(vec![]),
        _ => None,
    }
}

// Given a memory pointer instruction, compute the offset of indexed element
fn get_memory_offset(context: &Context, val: Value) -> Option<u64> {
    let sym = get_symbol(context, val)?;
    sym.get_type(context)
        .get_pointee_type(context)?
        .get_indexed_offset(context, &combine_indices(context, val)?)
}

// Can memory ranges [val1, val1+len1] and [val2, val2+len2] overlap?
// Conservatively returns true if cannot statically determine.
fn may_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let ((Some(sym1), Some(off1)), (Some(sym2), Some(off2))) =
        ((get_symbol(context, val1), get_memory_offset(context, val1)),
        (get_symbol(context, val2), get_memory_offset(context, val2))) else {
        return true
    };

    if sym1 != sym2 {
        return false;
    }

    // does off1 + len1 overlap with off2 + len2?
    (off1 <= off2 && (off1 + len1 > off2)) || (off2 <= off1 && (off2 + len2 > off1))
}
// Are memory ranges [val1, val1+len1] and [val2, val2+len2] exactly the same?
// Conservatively returns false if cannot statically determine.
fn must_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let ((Some(sym1), Some(off1)), (Some(sym2), Some(off2))) =
        ((get_symbol(context, val1), get_memory_offset(context, val1)),
        (get_symbol(context, val2), get_memory_offset(context, val2))) else {
        return false
    };

    if sym1 != sym2 {
        return false;
    }

    // does off1 + len1 overlap with off2 + len2?
    off1 == off2 && len1 == len2
}

fn pointee_size(context: &Context, ptr_val: Value) -> u64 {
    ptr_val
        .get_type(context)
        .unwrap()
        .get_pointee_type(context)
        .expect("Expected arg to be a pointer")
        .size_in_bytes(context)
}

/// Copy propagation of `memcpy`s within a block.
fn local_copy_prop(context: &mut Context, function: Function) -> Result<bool, IrError> {
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
        let sym = get_symbol(context, value).expect("Expected value representing a symbol");
        if let Some(copies) = src_to_copies.get_mut(&sym) {
            for copy in &*copies {
                let (src_ptr, copy_size) = match copy.get_instruction(context).unwrap() {
                    Instruction::MemCopyBytes {
                        dst_val_ptr: _,
                        src_val_ptr,
                        byte_len,
                    } => (*src_val_ptr, *byte_len),
                    Instruction::MemCopyVal {
                        dst_val_ptr: _,
                        src_val_ptr,
                    } => (*src_val_ptr, pointee_size(context, *src_val_ptr)),
                    _ => panic!("Unexpected copy instruction"),
                };
                if may_alias(context, value, len, src_ptr, copy_size) {
                    available_copies.remove(&copy);
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

    fn gen_new_copy(
        context: &Context,
        copy_inst: Value,
        dst_val_ptr: Value,
        src_val_ptr: Value,
        available_copies: &mut FxHashSet<Value>,
        src_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
        dest_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
    ) {
        if let Some(sym) = get_symbol(context, dst_val_ptr) {
            dest_to_copies
                .entry(sym)
                .and_modify(|set| {
                    set.insert(copy_inst);
                })
                .or_insert([copy_inst].into_iter().collect());
        }
        if let Some(sym) = get_symbol(context, src_val_ptr) {
            src_to_copies
                .entry(sym)
                .and_modify(|set| {
                    set.insert(copy_inst);
                })
                .or_insert([copy_inst].into_iter().collect());
        }
        available_copies.insert(copy_inst);
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
        inst: Value,
        src_val_ptr: Value,
        dest_to_copies: &mut FxHashMap<Symbol, FxHashSet<Value>>,
        replacements: &mut FxHashMap<Value, Replacement>,
    ) -> bool {
        // For every `memcpy` that src_val_ptr is a destination of,
        // check if we can do the load from the source of that memcpy.
        if let Some(src_sym) = get_symbol(context, src_val_ptr) {
            for memcpy in dest_to_copies
                .get(&src_sym)
                .iter()
                .map(|set| set.iter())
                .flatten()
            {
                let (dst_ptr_memcpy, src_ptr_memcpy, copy_len) =
                    deconstruct_memcpy(context, *memcpy);
                // If the location where we're loading from exactly matches the destination of
                // the memcpy, just load from the source pointer of the memcpy.
                if must_alias(
                    context,
                    src_val_ptr,
                    pointee_size(context, src_val_ptr),
                    dst_ptr_memcpy,
                    copy_len,
                ) {
                    // Replace src_val_ptr with src_ptr_memcpy.
                    replacements.insert(inst, Replacement::OldGep(src_ptr_memcpy));
                    return true;
                } else {
                    // if the memcpy copies the entire symbol, we could
                    // insert a new GEP from the source of the memcpy.
                    let dst_sym_size = pointee_size(context, dst_ptr_memcpy);
                    let src_sym_size = pointee_size(context, src_ptr_memcpy);
                    let is_full_sym_copy = dst_sym_size == src_sym_size && src_sym_size == copy_len;
                    if let (Some(memcpy_src_sym), true, Some(new_indices)) = (
                        get_symbol(context, src_ptr_memcpy),
                        is_full_sym_copy,
                        combine_indices(context, src_val_ptr),
                    ) {
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
        false
    }

    let mut modified = false;
    for block in function.block_iter(context) {
        loop {
            available_copies = FxHashSet::default();
            src_to_copies = FxHashMap::default();
            dest_to_copies = FxHashMap::default();

            // Replace the load/memcpy source pointer with something else.
            let mut replacements = FxHashMap::default();

            for inst in block.instruction_iter(context) {
                match inst.get_instruction(context).unwrap() {
                    Instruction::Call(_, args) => {
                        for arg in args {
                            let Some(arg_sym) = get_symbol(context, *arg) else { continue; };
                            let Some(arg_ty) = arg_sym.get_type(context).get_pointee_type(context) else { continue; };
                            kill_defined_symbol(
                                context,
                                *arg,
                                arg_ty.size_in_bytes(context),
                                &mut available_copies,
                                &mut src_to_copies,
                                &mut dest_to_copies,
                            );
                        }
                    }
                    Instruction::AsmBlock(_, args) => {
                        for arg in args {
                            let Some(arg_sym) = arg.initializer.and_then(|arg| get_symbol(context, arg)) else { continue; };
                            let Some(arg_ty) = arg_sym.get_type(context).get_pointee_type(context) else { continue; };
                            kill_defined_symbol(
                                context,
                                arg.initializer.unwrap(),
                                arg_ty.size_in_bytes(context),
                                &mut available_copies,
                                &mut src_to_copies,
                                &mut dest_to_copies,
                            );
                        }
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
                            inst,
                            src_val_ptr,
                            &mut dest_to_copies,
                            &mut replacements,
                        ) {
                            gen_new_copy(
                                context,
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

            // If we have any NewGep replacements, insert those new GEPs into the block.
            let mut new_insts = vec![];
            let replacements = replacements
                .into_iter()
                .map(|(load_inst, replacement)| {
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
                    (load_inst, replacement)
                })
                .collect::<Vec<_>>();

            block.prepend_instructions(context, new_insts);

            for replacement in &replacements {
                match replacement.0.get_instruction_mut(context) {
                    Some(Instruction::Load(ref mut src_val_ptr))
                    | Some(Instruction::MemCopyBytes {
                        ref mut src_val_ptr,
                        ..
                    })
                    | Some(Instruction::MemCopyVal {
                        ref mut src_val_ptr,
                        ..
                    }) => *src_val_ptr = replacement.1,
                    _ => panic!("Unexpected instruction type"),
                }
            }
            if !replacements.is_empty() {
                modified = true;
            } else {
                break;
            }
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
