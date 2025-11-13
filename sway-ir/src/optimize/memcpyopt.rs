//! Optimisations related to mem_copy.
//! - replace a `store` directly from a `load` with a `mem_copy_val`.

use indexmap::IndexMap;
use itertools::{Either, Itertools};
use rustc_hash::{FxHashMap, FxHashSet};
use sway_types::{FxIndexMap, FxIndexSet};

use crate::{
    get_gep_symbol, get_loaded_symbols, get_referred_symbol, get_referred_symbols,
    get_stored_symbols, memory_utils, AnalysisResults, Block, Context, EscapedSymbols,
    FuelVmInstruction, Function, InstOp, Instruction, InstructionInserter, IrError, LocalVar, Pass,
    PassMutability, ReferredSymbols, ScopedPass, Symbol, Type, Value, ValueDatum,
    ESCAPED_SYMBOLS_NAME,
};

pub const MEMCPYOPT_NAME: &str = "memcpyopt";

pub fn create_memcpyopt_pass() -> Pass {
    Pass {
        name: MEMCPYOPT_NAME,
        descr: "Optimizations related to MemCopy instructions",
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
    modified |= local_copy_prop_prememcpy(context, analyses, function)?;
    modified |= load_store_to_memcopy(context, function)?;
    modified |= local_copy_prop(context, analyses, function)?;

    Ok(modified)
}

fn local_copy_prop_prememcpy(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    struct InstInfo {
        // The block containing the instruction.
        block: Block,
        // Relative (use only for comparison) position of instruction in `block`.
        pos: usize,
    }

    // If the analysis result is incomplete we cannot do any safe optimizations here.
    // Calculating the candidates below relies on complete result of an escape analysis.
    let escaped_symbols = match analyses.get_analysis_result(function) {
        EscapedSymbols::Complete(syms) => syms,
        EscapedSymbols::Incomplete(_) => return Ok(false),
    };

    // All instructions that load from the `Symbol`.
    let mut loads_map = FxHashMap::<Symbol, Vec<Value>>::default();
    // All instructions that store to the `Symbol`.
    let mut stores_map = FxHashMap::<Symbol, Vec<Value>>::default();
    // All load and store instructions.
    let mut instr_info_map = FxHashMap::<Value, InstInfo>::default();

    for (pos, (block, inst)) in function.instruction_iter(context).enumerate() {
        let info = || InstInfo { block, pos };
        let inst_e = inst.get_instruction(context).unwrap();
        match inst_e {
            Instruction {
                op: InstOp::Load(src_val_ptr),
                ..
            } => {
                if let Some(local) = get_referred_symbol(context, *src_val_ptr) {
                    loads_map
                        .entry(local)
                        .and_modify(|loads| loads.push(inst))
                        .or_insert(vec![inst]);
                    instr_info_map.insert(inst, info());
                }
            }
            Instruction {
                op: InstOp::Store { dst_val_ptr, .. },
                ..
            } => {
                if let Some(local) = get_referred_symbol(context, *dst_val_ptr) {
                    stores_map
                        .entry(local)
                        .and_modify(|stores| stores.push(inst))
                        .or_insert(vec![inst]);
                    instr_info_map.insert(inst, info());
                }
            }
            _ => (),
        }
    }

    let mut to_delete = FxHashSet::<Value>::default();
    // Candidates for replacements. The map's key `Symbol` is the
    // destination `Symbol` that can be replaced with the
    // map's value `Symbol`, the source.
    // Replacement is possible (among other criteria explained below)
    // only if the Store of the source is the only storing to the destination.
    let candidates: FxHashMap<Symbol, Symbol> = function
        .instruction_iter(context)
        .enumerate()
        .filter_map(|(pos, (block, instr_val))| {
            // 1. Go through all the Store instructions whose source is
            // a Load instruction...
            instr_val
                .get_instruction(context)
                .and_then(|instr| {
                    // Is the instruction a Store?
                    if let Instruction {
                        op:
                            InstOp::Store {
                                dst_val_ptr,
                                stored_val,
                            },
                        ..
                    } = instr
                    {
                        get_gep_symbol(context, *dst_val_ptr).and_then(|dst_local| {
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
                    if let Instruction {
                        op: InstOp::Load(src_val_ptr),
                        ..
                    } = src_instr
                    {
                        get_gep_symbol(context, *src_val_ptr)
                            .map(|src_local| (stored_val, dst_local, src_local))
                    } else {
                        None
                    }
                })
                .and_then(|(src_load, dst_local, src_local)| {
                    // 2. ... and pick the (dest_local, src_local) pairs that fulfill the
                    //    below criteria, in other words, where `dest_local` can be
                    //    replaced with `src_local`.
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
                        // We don't deal with symbols that escape.
                        || escaped_symbols.contains(&dst_local)
                        || escaped_symbols.contains(&src_local)
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

    // If we have A replaces B and B replaces C, then A must replace C also.
    // Recursively searches for the final replacement for the `local`.
    // Returns `None` if the `local` cannot be replaced.
    fn get_replace_with(candidates: &FxHashMap<Symbol, Symbol>, local: &Symbol) -> Option<Symbol> {
        candidates
            .get(local)
            .map(|replace_with| get_replace_with(candidates, replace_with).unwrap_or(*replace_with))
    }

    // If the source is an Arg, we replace uses of destination with Arg.
    // Otherwise (`get_local`), we replace the local symbol in-place.
    enum ReplaceWith {
        InPlaceLocal(LocalVar),
        Value(Value),
    }

    // Because we can't borrow context for both iterating and replacing, do it in 2 steps.
    // `replaces` are the original GetLocal instructions with the corresponding replacements
    // of their arguments.
    let replaces: Vec<_> = function
        .instruction_iter(context)
        .filter_map(|(_block, value)| match value.get_instruction(context) {
            Some(Instruction {
                op: InstOp::GetLocal(local),
                ..
            }) => get_replace_with(&candidates, &Symbol::Local(*local)).map(|replace_with| {
                (
                    value,
                    match replace_with {
                        Symbol::Local(local) => ReplaceWith::InPlaceLocal(local),
                        Symbol::Arg(ba) => {
                            ReplaceWith::Value(ba.block.get_arg(context, ba.idx).unwrap())
                        }
                    },
                )
            }),
            _ => None,
        })
        .collect();

    let mut value_replace = FxHashMap::<Value, Value>::default();
    for (value, replace_with) in replaces.into_iter() {
        match replace_with {
            ReplaceWith::InPlaceLocal(replacement_var) => {
                let Some(&Instruction {
                    op: InstOp::GetLocal(redundant_var),
                    parent,
                }) = value.get_instruction(context)
                else {
                    panic!("earlier match now fails");
                };
                if redundant_var.is_mutable(context) {
                    replacement_var.set_mutable(context, true);
                }
                value.replace(
                    context,
                    ValueDatum::Instruction(Instruction {
                        op: InstOp::GetLocal(replacement_var),
                        parent,
                    }),
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

// Deconstruct a memcpy into (dst_val_ptr, src_val_ptr, copy_len).
fn deconstruct_memcpy(context: &Context, inst: Value) -> Option<(Value, Value, u64)> {
    match inst.get_instruction(context).unwrap() {
        Instruction {
            op:
                InstOp::MemCopyBytes {
                    dst_val_ptr,
                    src_val_ptr,
                    byte_len,
                },
            ..
        } => Some((*dst_val_ptr, *src_val_ptr, *byte_len)),
        Instruction {
            op:
                InstOp::MemCopyVal {
                    dst_val_ptr,
                    src_val_ptr,
                },
            ..
        } => Some((
            *dst_val_ptr,
            *src_val_ptr,
            memory_utils::pointee_size(context, *dst_val_ptr),
        )),
        _ => None,
    }
}

/// Copy propagation of `memcpy`s within a block.
fn local_copy_prop(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // If the analysis result is incomplete we cannot do any safe optimizations here.
    // The `gen_new_copy` and `process_load` functions below rely on the fact that the
    // analyzed symbols do not escape, something we cannot guarantee in case of
    // an incomplete collection of escaped symbols.
    let escaped_symbols = match analyses.get_analysis_result(function) {
        EscapedSymbols::Complete(syms) => syms,
        EscapedSymbols::Incomplete(_) => return Ok(false),
    };

    // Currently (as we scan a block) available `memcpy`s.
    let mut available_copies: FxHashSet<Value>;
    // Map a symbol to the available `memcpy`s of which it's a source.
    let mut src_to_copies: FxIndexMap<Symbol, FxIndexSet<Value>>;
    // Map a symbol to the available `memcpy`s of which it's a destination.
    // (multiple `memcpy`s for the same destination may be available when
    // they are partial / field writes, and don't alias).
    let mut dest_to_copies: FxIndexMap<Symbol, FxIndexSet<Value>>;

    // If a value (symbol) is found to be defined, remove it from our tracking.
    fn kill_defined_symbol(
        context: &Context,
        value: Value,
        len: u64,
        available_copies: &mut FxHashSet<Value>,
        src_to_copies: &mut FxIndexMap<Symbol, FxIndexSet<Value>>,
        dest_to_copies: &mut FxIndexMap<Symbol, FxIndexSet<Value>>,
    ) {
        match get_referred_symbols(context, value) {
            ReferredSymbols::Complete(rs) => {
                for sym in rs {
                    if let Some(copies) = src_to_copies.get_mut(&sym) {
                        for copy in &*copies {
                            let (_, src_ptr, copy_size) = deconstruct_memcpy(context, *copy)
                                .expect("Expected copy instruction");
                            if memory_utils::may_alias(context, value, len, src_ptr, copy_size) {
                                available_copies.remove(copy);
                            }
                        }
                    }
                    if let Some(copies) = dest_to_copies.get_mut(&sym) {
                        for copy in &*copies {
                            let (dest_ptr, copy_size) = match copy.get_instruction(context).unwrap()
                            {
                                Instruction {
                                    op:
                                        InstOp::MemCopyBytes {
                                            dst_val_ptr,
                                            src_val_ptr: _,
                                            byte_len,
                                        },
                                    ..
                                } => (*dst_val_ptr, *byte_len),
                                Instruction {
                                    op:
                                        InstOp::MemCopyVal {
                                            dst_val_ptr,
                                            src_val_ptr: _,
                                        },
                                    ..
                                } => (
                                    *dst_val_ptr,
                                    memory_utils::pointee_size(context, *dst_val_ptr),
                                ),
                                _ => panic!("Unexpected copy instruction"),
                            };
                            if memory_utils::may_alias(context, value, len, dest_ptr, copy_size) {
                                available_copies.remove(copy);
                            }
                        }
                    }
                }
                // Update src_to_copies and dest_to_copies to remove every copy not in available_copies.
                src_to_copies.retain(|_, copies| {
                    copies.retain(|copy| available_copies.contains(copy));
                    !copies.is_empty()
                });
                dest_to_copies.retain(|_, copies| {
                    copies.retain(|copy| available_copies.contains(copy));
                    !copies.is_empty()
                });
            }
            ReferredSymbols::Incomplete(_) => {
                // The only safe thing we can do is to clear all information.
                available_copies.clear();
                src_to_copies.clear();
                dest_to_copies.clear();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_new_copy(
        context: &Context,
        escaped_symbols: &FxHashSet<Symbol>,
        copy_inst: Value,
        dst_val_ptr: Value,
        src_val_ptr: Value,
        available_copies: &mut FxHashSet<Value>,
        src_to_copies: &mut FxIndexMap<Symbol, FxIndexSet<Value>>,
        dest_to_copies: &mut FxIndexMap<Symbol, FxIndexSet<Value>>,
    ) {
        if let (Some(dst_sym), Some(src_sym)) = (
            get_gep_symbol(context, dst_val_ptr),
            get_gep_symbol(context, src_val_ptr),
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
        escaped_symbols: &FxHashSet<Symbol>,
        inst: Value,
        src_val_ptr: Value,
        dest_to_copies: &FxIndexMap<Symbol, FxIndexSet<Value>>,
        replacements: &mut FxHashMap<Value, (Value, Replacement)>,
    ) -> bool {
        // For every `memcpy` that src_val_ptr is a destination of,
        // check if we can do the load from the source of that memcpy.
        if let Some(src_sym) = get_referred_symbol(context, src_val_ptr) {
            if escaped_symbols.contains(&src_sym) {
                return false;
            }
            for memcpy in dest_to_copies
                .get(&src_sym)
                .iter()
                .flat_map(|set| set.iter())
            {
                let (dst_ptr_memcpy, src_ptr_memcpy, copy_len) =
                    deconstruct_memcpy(context, *memcpy).expect("Expected copy instruction");
                // If the location where we're loading from exactly matches the destination of
                // the memcpy, just load from the source pointer of the memcpy.
                // TODO: In both the arms below, we check that the pointer type
                // matches. This isn't really needed as the copy happens and the
                // data we want is safe to access. But we just don't know how to
                // generate the right GEP always. So that's left for another day.
                if memory_utils::must_alias(
                    context,
                    src_val_ptr,
                    memory_utils::pointee_size(context, src_val_ptr),
                    dst_ptr_memcpy,
                    copy_len,
                ) {
                    // Replace src_val_ptr with src_ptr_memcpy.
                    if src_val_ptr.get_type(context) == src_ptr_memcpy.get_type(context) {
                        replacements
                            .insert(inst, (src_val_ptr, Replacement::OldGep(src_ptr_memcpy)));
                        return true;
                    }
                } else {
                    // if the memcpy copies the entire symbol, we could
                    // insert a new GEP from the source of the memcpy.
                    if let (Some(memcpy_src_sym), Some(memcpy_dst_sym), Some(new_indices)) = (
                        get_gep_symbol(context, src_ptr_memcpy),
                        get_gep_symbol(context, dst_ptr_memcpy),
                        memory_utils::combine_indices(context, src_val_ptr),
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
                            && memcpy_dst_sym_type.size(context).in_bytes() == copy_len
                        {
                            replacements.insert(
                                inst,
                                (
                                    src_val_ptr,
                                    Replacement::NewGep(ReplGep {
                                        base: memcpy_src_sym,
                                        elem_ptr_ty: src_val_ptr.get_type(context).unwrap(),
                                        indices: new_indices,
                                    }),
                                ),
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
            src_to_copies = IndexMap::default();
            dest_to_copies = IndexMap::default();

            // Replace the load/memcpy source pointer with something else.
            let mut replacements = FxHashMap::default();

            fn kill_escape_args(
                context: &Context,
                args: &Vec<Value>,
                available_copies: &mut FxHashSet<Value>,
                src_to_copies: &mut FxIndexMap<Symbol, FxIndexSet<Value>>,
                dest_to_copies: &mut FxIndexMap<Symbol, FxIndexSet<Value>>,
            ) {
                for arg in args {
                    match get_referred_symbols(context, *arg) {
                        ReferredSymbols::Complete(rs) => {
                            let max_size = rs
                                .iter()
                                .filter_map(|sym| {
                                    sym.get_type(context)
                                        .get_pointee_type(context)
                                        .map(|pt| pt.size(context).in_bytes())
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
                        ReferredSymbols::Incomplete(_) => {
                            // The only safe thing we can do is to clear all information.
                            available_copies.clear();
                            src_to_copies.clear();
                            dest_to_copies.clear();

                            break;
                        }
                    }
                }
            }

            for inst in block.instruction_iter(context) {
                match inst.get_instruction(context).unwrap() {
                    Instruction {
                        op: InstOp::Call(callee, args),
                        ..
                    } => {
                        let (immutable_args, mutable_args): (Vec<_>, Vec<_>) =
                            args.iter().enumerate().partition_map(|(arg_idx, arg)| {
                                if callee.is_arg_immutable(context, arg_idx) {
                                    Either::Left(*arg)
                                } else {
                                    Either::Right(*arg)
                                }
                            });
                        // whichever args may get mutated, we kill them.
                        kill_escape_args(
                            context,
                            &mutable_args,
                            &mut available_copies,
                            &mut src_to_copies,
                            &mut dest_to_copies,
                        );
                        // args that aren't mutated can be treated as a "load" (for the purposes
                        // of optimization).
                        for arg in immutable_args {
                            process_load(
                                context,
                                escaped_symbols,
                                inst,
                                arg,
                                &dest_to_copies,
                                &mut replacements,
                            );
                        }
                    }
                    Instruction {
                        op: InstOp::AsmBlock(_, args),
                        ..
                    } => {
                        let args = args.iter().filter_map(|arg| arg.initializer).collect();
                        kill_escape_args(
                            context,
                            &args,
                            &mut available_copies,
                            &mut src_to_copies,
                            &mut dest_to_copies,
                        );
                    }
                    Instruction {
                        op: InstOp::IntToPtr(_, _),
                        ..
                    } => {
                        // The only safe thing we can do is to clear all information.
                        available_copies.clear();
                        src_to_copies.clear();
                        dest_to_copies.clear();
                    }
                    Instruction {
                        op: InstOp::Load(src_val_ptr),
                        ..
                    } => {
                        process_load(
                            context,
                            escaped_symbols,
                            inst,
                            *src_val_ptr,
                            &dest_to_copies,
                            &mut replacements,
                        );
                    }
                    Instruction {
                        op: InstOp::MemCopyBytes { .. } | InstOp::MemCopyVal { .. },
                        ..
                    } => {
                        let (dst_val_ptr, src_val_ptr, copy_len) =
                            deconstruct_memcpy(context, inst).expect("Expected copy instruction");
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
                            &dest_to_copies,
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
                    Instruction {
                        op:
                            InstOp::Store {
                                dst_val_ptr,
                                stored_val: _,
                            },
                        ..
                    } => {
                        kill_defined_symbol(
                            context,
                            *dst_val_ptr,
                            memory_utils::pointee_size(context, *dst_val_ptr),
                            &mut available_copies,
                            &mut src_to_copies,
                            &mut dest_to_copies,
                        );
                    }
                    Instruction {
                        op:
                            InstOp::FuelVm(
                                FuelVmInstruction::WideBinaryOp { result, .. }
                                | FuelVmInstruction::WideUnaryOp { result, .. }
                                | FuelVmInstruction::WideModularOp { result, .. }
                                | FuelVmInstruction::StateLoadQuadWord {
                                    load_val: result, ..
                                },
                            ),
                        ..
                    } => {
                        kill_defined_symbol(
                            context,
                            *result,
                            memory_utils::pointee_size(context, *result),
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
                    let (to_replace, replacement) = match replacement {
                        (to_replace, Replacement::OldGep(v)) => (to_replace, v),
                        (
                            to_replace,
                            Replacement::NewGep(ReplGep {
                                base,
                                elem_ptr_ty,
                                indices,
                            }),
                        ) => {
                            let base = match base {
                                Symbol::Local(local) => {
                                    let base = Value::new_instruction(
                                        context,
                                        block,
                                        InstOp::GetLocal(local),
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
                                block,
                                InstOp::GetElemPtr {
                                    base,
                                    elem_ptr_ty,
                                    indices,
                                },
                            );
                            new_insts.push(v);
                            (to_replace, v)
                        }
                    };
                    match inst.get_instruction_mut(context) {
                        Some(Instruction {
                            op: InstOp::Load(ref mut src_val_ptr),
                            ..
                        })
                        | Some(Instruction {
                            op:
                                InstOp::MemCopyBytes {
                                    ref mut src_val_ptr,
                                    ..
                                },
                            ..
                        })
                        | Some(Instruction {
                            op:
                                InstOp::MemCopyVal {
                                    ref mut src_val_ptr,
                                    ..
                                },
                            ..
                        }) => {
                            assert!(to_replace == *src_val_ptr);
                            *src_val_ptr = replacement
                        }
                        Some(Instruction {
                            op: InstOp::Call(_callee, args),
                            ..
                        }) => {
                            for arg in args {
                                if *arg == to_replace {
                                    *arg = replacement;
                                }
                            }
                        }
                        _ => panic!("Unexpected instruction type"),
                    }
                }
                new_insts.push(inst);
            }

            // Replace the basic block contents with what we just built.
            block.take_body(context, new_insts);
        }
    }

    Ok(modified)
}

struct Candidate {
    load_val: Value,
    store_val: Value,
    dst_ptr: Value,
    src_ptr: Value,
}

enum CandidateKind {
    /// If aggregates are clobbered b/w a load and the store, we still need to,
    /// for correctness (because asmgen cannot handle aggregate loads and stores)
    /// do the memcpy. So we insert a memcpy to a temporary stack location right after
    /// the load, and memcpy it to the store pointer at the point of store.
    ClobberedNoncopyType(Candidate),
    NonClobbered(Candidate),
}

/// Starting backwards from `end_inst`, till we reach `start_inst` or the entry block,
/// is `scrutiny_ptr` (or an alias of it) stored to (i.e., clobbered)?
/// Also checks that there is no overlap (common symbols) between
/// `no_overlap_ptr` and `scrutiny_ptr`.
fn is_clobbered(
    context: &Context,
    start_inst: &Value,
    end_inst: &Value,
    no_overlap_ptr: &Value,
    scrutiny_ptr: &Value,
) -> bool {
    let end_block = end_inst.get_instruction(context).unwrap().parent;
    let entry_block = end_block.get_function(context).get_entry_block(context);

    let mut iter = end_block
        .instruction_iter(context)
        .rev()
        .skip_while(|i| i != end_inst);
    assert!(iter.next().unwrap() == *end_inst);

    let ReferredSymbols::Complete(scrutiny_symbols) = get_referred_symbols(context, *scrutiny_ptr)
    else {
        return true;
    };

    let ReferredSymbols::Complete(no_overlap_symbols) =
        get_referred_symbols(context, *no_overlap_ptr)
    else {
        return true;
    };

    // If the two pointers may have an overlap, we'll end up generating a mcp
    // with overlapping source/destination which is not allowed.
    if scrutiny_symbols
        .intersection(&no_overlap_symbols)
        .next()
        .is_some()
    {
        return true;
    }

    // Scan backwards till we encounter start_val, checking if
    // any store aliases with scrutiny_ptr.
    let mut worklist: Vec<(Block, Box<dyn Iterator<Item = Value>>)> =
        vec![(end_block, Box::new(iter))];
    let mut visited = FxHashSet::default();
    'next_job: while let Some((block, iter)) = worklist.pop() {
        visited.insert(block);
        for inst in iter {
            if inst == *start_inst || inst == *end_inst {
                // We don't need to go beyond either start_inst or end_inst.
                continue 'next_job;
            }
            let stored_syms = get_stored_symbols(context, inst);
            if let ReferredSymbols::Complete(syms) = stored_syms {
                if syms.iter().any(|sym| scrutiny_symbols.contains(sym)) {
                    return true;
                }
            } else {
                return true;
            }
        }

        if entry_block == block {
            // We've reached the entry block. If any of the scrutiny_symbols
            // is an argument, then we consider it clobbered.
            if scrutiny_symbols
                .iter()
                .any(|sym| matches!(sym, Symbol::Arg(_)))
            {
                return true;
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

// This is a copy of sway_core::asm_generation::fuel::fuel_asm_builder::FuelAsmBuilder::is_copy_type.
fn is_copy_type(ty: &Type, context: &Context) -> bool {
    ty.is_unit(context)
        || ty.is_never(context)
        || ty.is_bool(context)
        || ty.is_ptr(context)
        || ty.get_uint_width(context).map(|x| x < 256).unwrap_or(false)
}

fn load_store_to_memcopy(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find any `store`s of `load`s.  These can be replaced with `mem_copy` and are especially
    // important for non-copy types on architectures which don't support loading them.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(_, store_instr_val)| {
            store_instr_val
                .get_instruction(context)
                .and_then(|instr| {
                    // Is the instruction a Store?
                    if let Instruction {
                        op:
                            InstOp::Store {
                                dst_val_ptr,
                                stored_val,
                            },
                        ..
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
                    if let Instruction {
                        op: InstOp::Load(src_val_ptr),
                        ..
                    } = src_instr
                    {
                        Some(Candidate {
                            load_val: src_instr_val,
                            store_val: store_instr_val,
                            dst_ptr: *dst_val_ptr,
                            src_ptr: *src_val_ptr,
                        })
                    } else {
                        None
                    }
                })
                .and_then(|candidate @ Candidate { dst_ptr, .. }| {
                    // Check that there's no path from load_val to store_val that might overwrite src_ptr.
                    if !is_clobbered(
                        context,
                        &candidate.load_val,
                        &candidate.store_val,
                        &candidate.dst_ptr,
                        &candidate.src_ptr,
                    ) {
                        Some(CandidateKind::NonClobbered(candidate))
                    } else if !is_copy_type(&dst_ptr.match_ptr_type(context).unwrap(), context) {
                        Some(CandidateKind::ClobberedNoncopyType(candidate))
                    } else {
                        None
                    }
                })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    for candidate in candidates {
        match candidate {
            CandidateKind::ClobberedNoncopyType(Candidate {
                load_val,
                store_val,
                dst_ptr,
                src_ptr,
            }) => {
                let load_block = load_val.get_instruction(context).unwrap().parent;
                let temp = function.new_unique_local_var(
                    context,
                    "__aggr_memcpy_0".into(),
                    src_ptr.match_ptr_type(context).unwrap(),
                    None,
                    true,
                );
                let temp_local =
                    Value::new_instruction(context, load_block, InstOp::GetLocal(temp));
                let to_temp = Value::new_instruction(
                    context,
                    load_block,
                    InstOp::MemCopyVal {
                        dst_val_ptr: temp_local,
                        src_val_ptr: src_ptr,
                    },
                );
                let mut inserter = InstructionInserter::new(
                    context,
                    load_block,
                    crate::InsertionPosition::After(load_val),
                );
                inserter.insert_slice(&[temp_local, to_temp]);

                let store_block = store_val.get_instruction(context).unwrap().parent;
                let mem_copy_val = Value::new_instruction(
                    context,
                    store_block,
                    InstOp::MemCopyVal {
                        dst_val_ptr: dst_ptr,
                        src_val_ptr: temp_local,
                    },
                );
                store_block.replace_instruction(context, store_val, mem_copy_val, true)?;
            }
            CandidateKind::NonClobbered(Candidate {
                dst_ptr: dst_val_ptr,
                src_ptr: src_val_ptr,
                store_val,
                ..
            }) => {
                let store_block = store_val.get_instruction(context).unwrap().parent;
                let mem_copy_val = Value::new_instruction(
                    context,
                    store_block,
                    InstOp::MemCopyVal {
                        dst_val_ptr,
                        src_val_ptr,
                    },
                );
                store_block.replace_instruction(context, store_val, mem_copy_val, true)?;
            }
        }
    }

    Ok(true)
}

pub const MEMCPYPROP_REVERSE_NAME: &str = "memcpyprop_reverse";

pub fn create_memcpyprop_reverse_pass() -> Pass {
    Pass {
        name: MEMCPYPROP_REVERSE_NAME,
        descr: "Copy propagation of MemCpy instructions",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(copy_prop_reverse)),
    }
}

/// Copy propagation of `memcpy`s, replacing source with destination.
fn copy_prop_reverse(
    context: &mut Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;

    // let's first compute the definitions and uses of every symbol.
    let mut stores_map: FxHashMap<Symbol, Vec<Value>> = FxHashMap::default();
    let mut loads_map: FxHashMap<Symbol, Vec<Value>> = FxHashMap::default();
    for (_block, instr_val) in function.instruction_iter(context) {
        let stored_syms = get_stored_symbols(context, instr_val);
        let stored_syms = match stored_syms {
            ReferredSymbols::Complete(syms) => syms,
            ReferredSymbols::Incomplete(_) => return Ok(false),
        };
        let loaded_syms = get_loaded_symbols(context, instr_val);
        let loaded_syms = match loaded_syms {
            ReferredSymbols::Complete(syms) => syms,
            ReferredSymbols::Incomplete(_) => return Ok(false),
        };
        for sym in stored_syms {
            stores_map.entry(sym).or_default().push(instr_val);
        }
        for sym in loaded_syms {
            loads_map.entry(sym).or_default().push(instr_val);
        }
    }

    let mut candidates = vec![];

    for (_block, inst) in function.instruction_iter(context) {
        let Some((dst_ptr, src_ptr, byte_len)) = deconstruct_memcpy(context, inst) else {
            continue;
        };

        if dst_ptr.get_type(context) != src_ptr.get_type(context) {
            continue;
        }

        // We can replace the source of this memcpy with the destination
        // if:
        // 1. All uses of the destination symbol are dominated by this memcpy.
        // 2. All uses of the source symbol are dominated by this memcpy.

        let dst_sym = match get_referred_symbols(context, dst_ptr) {
            ReferredSymbols::Complete(syms) if syms.len() == 1 => syms.into_iter().next().unwrap(),
            _ => continue,
        };
        let src_sym = match get_referred_symbols(context, src_ptr) {
            ReferredSymbols::Complete(syms) if syms.len() == 1 => syms.into_iter().next().unwrap(),
            _ => continue,
        };

        // We don't deal with partial memcpys
        if dst_sym
            .get_type(context)
            .get_pointee_type(context)
            .expect("All symbols must be pointer types")
            .size(context)
            .in_bytes()
            != byte_len
        {
            continue;
        }

        // For every use of the source symbol:
        //   starting from the use, walk backwards till we reach this memcpy,
        //   checking if there's a store to an alias of the destination symbol in that path.
        let source_uses_not_clobbered = loads_map
            .get(&src_sym)
            .map(|uses| {
                uses.iter().all(|use_val: &Value| {
                    *use_val == inst || !is_clobbered(context, &inst, use_val, &src_ptr, &dst_ptr)
                })
            })
            .unwrap_or(true);

        // For every use of the destination symbol:
        //   starting from the use, walk backwards till we reach this memcpy,
        //   checking if there's a store to an alias of the source symbol in that path.
        let destination_uses_not_clobbered = loads_map
            .get(&dst_sym)
            .map(|uses| {
                uses.iter()
                    .all(|use_val| !is_clobbered(context, &inst, use_val, &dst_ptr, &src_ptr))
            })
            .unwrap_or(true);

        if source_uses_not_clobbered && destination_uses_not_clobbered {
            candidates.push((inst, dst_sym, src_sym));
        }
    }

    if candidates.is_empty() {
        return Ok(false);
    }

    let mut to_delete: FxHashSet<Value> = FxHashSet::default();
    let mut src_to_dst: FxHashMap<Symbol, Symbol> = FxHashMap::default();

    for (inst, dst_sym, src_sym) in candidates {
        match src_sym {
            Symbol::Arg(_) => {
                // Args are mostly copied to locals before actually being used.
                // So we don't handle them for now. Handling them would require
                // handling more instructions where they can be used, which probably
                // isn't worth it.
                continue;
            }
            Symbol::Local(local) => {
                if local.get_initializer(context).is_some() {
                    // If the source is a local and it has an initializer, we run into trouble
                    // 1. If the destination (after transitive closure below) is not a local,
                    //    we cannot initialize it with the source's initializer.
                    // 2. If the destination is a local, but it already has an initializer (by itself
                    //    or by another source in the chain), we cannot initialize it with this initializer.
                    continue;
                }
                match src_to_dst.entry(src_sym) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(dst_sym);
                    }
                    std::collections::hash_map::Entry::Occupied(e) => {
                        if *e.get() != dst_sym {
                            // src_sym is copied to two different dst_syms. We cannot optimize this.
                            continue;
                        }
                    }
                }
                to_delete.insert(inst);
            }
        }
    }

    // Take a transitive closure of src_to_dst.
    {
        let mut changed = true;
        let mut cycle_detected = false;
        while changed {
            changed = false;
            src_to_dst.clone().iter().for_each(|(src, dst)| {
                if let Some(next_dst) = src_to_dst.get(dst) {
                    // Cycle detection
                    if *next_dst == *src {
                        cycle_detected = true;
                        return;
                    }
                    src_to_dst.insert(*src, *next_dst);
                    changed = true;
                }
            });
        }
        if cycle_detected {
            // We cannot optimize in presence of cycles.
            return Ok(modified);
        }
    }

    let mut repl_locals = vec![];
    let mut value_replacements = FxHashMap::default();

    // Gather the get_local instructions that need to be replaced.
    for (_block, inst) in function.instruction_iter(context) {
        match inst.get_instruction(context).cloned().unwrap() {
            Instruction {
                op: InstOp::GetLocal(sym),
                parent,
                ..
            } => {
                if let Some(dst) = src_to_dst.get(&Symbol::Local(sym)) {
                    let sym_type = sym.get_type(context);
                    let dst_type = dst.get_type(context);

                    if sym_type.eq(context, &dst_type) {
                        repl_locals.push((inst, *dst));
                    } else {
                        let original_ptr = match dst {
                            Symbol::Local(..) => todo!(),
                            Symbol::Arg(block_argument) => block_argument.as_value(context),
                        };

                        let cast_ptr = InstOp::CastPtr(original_ptr, sym_type);
                        value_replacements.insert(inst, (parent, cast_ptr));
                    }
                }
            }
            _ => {
                // Any access to a local begins with a GetLocal, we can ignore the rest
                // (unless we support Symbol::Arg above).
            }
        }
    }

    if repl_locals.is_empty() && value_replacements.is_empty() {
        return Ok(modified);
    }

    modified = true;

    let mut value_replacements = value_replacements
        .into_iter()
        .map(|(old, (block, instruction))| {
            let v = Value::new_instruction(context, block, instruction);

            let mut inserter =
                InstructionInserter::new(context, block, crate::InsertionPosition::Before(old));
            inserter.insert(v);

            (old, v)
        })
        .collect::<FxHashMap<Value, Value>>();

    for (to_repl, repl_with) in repl_locals {
        let Instruction {
            op: InstOp::GetLocal(sym),
            ..
        } = to_repl.get_instruction_mut(context).unwrap()
        else {
            panic!("Expected GetLocal instruction");
        };
        match repl_with {
            Symbol::Local(dst_local) => {
                // We just modify this GetLocal in-place.
                *sym = dst_local;
            }
            Symbol::Arg(arg) => {
                // The get_local needs to be replaced with the right argument Value.
                value_replacements.insert(to_repl, arg.as_value(context));
            }
        }
    }

    // Replace get_locals with the right values.
    function.replace_values(context, &value_replacements, None);

    // In instances such as
    //        (1) b <- a
    //        /         \
    // (2) x <- b    (3):  x <- a
    // when we decide to eliminate (1) and (2), i.e., both `b` and `a` end up
    // being replaced by `x`, (3) will end up becoming `x <- x`. We need to
    // clean these up.
    for (_, inst) in function.instruction_iter(context) {
        let Some((dst_ptr, src_ptr, _byte_len)) = deconstruct_memcpy(context, inst) else {
            continue;
        };

        let dst_sym = match get_referred_symbols(context, dst_ptr) {
            ReferredSymbols::Complete(syms) if syms.len() == 1 => syms.into_iter().next().unwrap(),
            _ => continue,
        };
        let src_sym = match get_referred_symbols(context, src_ptr) {
            ReferredSymbols::Complete(syms) if syms.len() == 1 => syms.into_iter().next().unwrap(),
            _ => continue,
        };

        if dst_sym == src_sym {
            to_delete.insert(inst);
        }
    }

    function.remove_instructions(context, |v| to_delete.contains(&v));

    Ok(modified)
}
