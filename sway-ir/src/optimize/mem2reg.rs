use indexmap::IndexMap;
/// Promote local memory to SSA registers.
/// This pass is essentially SSA construction. A good readable reference is:
/// https://www.cs.princeton.edu/~appel/modern/c/
/// We use block arguments instead of explicit PHI nodes. Conceptually,
/// they are both the same.
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use sway_utils::mapped_stack::MappedStack;

use crate::{
    AnalysisResults, Block, BranchToWithArgs, Constant, Context, DomFronts, DomTree, Function,
    InstOp, Instruction, IrError, LocalVar, Pass, PassMutability, PostOrder, ScopedPass, Type,
    Value, ValueDatum, DOMINATORS_NAME, DOM_FRONTS_NAME, POSTORDER_NAME,
};

pub const MEM2REG_NAME: &str = "mem2reg";

pub fn create_mem2reg_pass() -> Pass {
    Pass {
        name: MEM2REG_NAME,
        descr: "Promotion of memory to SSA registers",
        deps: vec![POSTORDER_NAME, DOMINATORS_NAME, DOM_FRONTS_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(promote_to_registers)),
    }
}

// Check if a value is a valid (for our optimization) local pointer
fn get_validate_local_var(
    context: &Context,
    function: &Function,
    val: &Value,
) -> Option<(String, LocalVar)> {
    match context.values[val.0].value {
        ValueDatum::Instruction(Instruction {
            op: InstOp::GetLocal(local_var),
            ..
        }) => {
            let name = function.lookup_local_name(context, &local_var);
            name.map(|name| (name.clone(), local_var))
        }
        _ => None,
    }
}

fn is_promotable_type(context: &Context, ty: Type) -> bool {
    ty.is_unit(context)
        || ty.is_bool(context)
        || (ty.is_uint(context) && ty.get_uint_width(context).unwrap() <= 64)
}

// Returns those locals that can be promoted to SSA registers.
fn filter_usable_locals(context: &mut Context, function: &Function) -> HashSet<String> {
    // The size of an SSA register is target specific.  Here we're going to just stick with atomic
    // types which can fit in 64-bits.
    let mut locals: HashSet<String> = function
        .locals_iter(context)
        .filter_map(|(name, var)| {
            let ty = var.get_inner_type(context);
            is_promotable_type(context, ty).then_some(name.clone())
        })
        .collect();

    for (_, inst) in function.instruction_iter(context) {
        match context.values[inst.0].value {
            ValueDatum::Instruction(Instruction {
                op: InstOp::Load(_),
                ..
            }) => {}
            ValueDatum::Instruction(Instruction {
                op:
                    InstOp::Store {
                        dst_val_ptr: _,
                        stored_val,
                    },
                ..
            }) => {
                // Make sure that a local ('s address) isn't stored.
                if let Some((local, _)) = get_validate_local_var(context, function, &stored_val) {
                    locals.remove(&local);
                }
            }
            _ => {
                // Make sure that no local escapes into instructions we don't understand.
                let operands = inst.get_instruction(context).unwrap().op.get_operands();
                for opd in operands {
                    if let Some((local, ..)) = get_validate_local_var(context, function, &opd) {
                        locals.remove(&local);
                    }
                }
            }
        }
    }
    locals
}

// For each block, compute the set of locals that are live-in.
// TODO: Use rustc_index::bit_set::ChunkedBitSet by mapping local names to indices.
//       This will allow more efficient set operations.
pub fn compute_livein(
    context: &mut Context,
    function: &Function,
    po: &PostOrder,
    locals: &HashSet<String>,
) -> FxHashMap<Block, HashSet<String>> {
    let mut result = FxHashMap::<Block, HashSet<String>>::default();
    for block in &po.po_to_block {
        result.insert(*block, HashSet::<String>::default());
    }

    let mut changed = true;
    while changed {
        changed = false;
        for block in &po.po_to_block {
            // we begin by unioning the liveins at successor blocks.
            let mut cur_live = HashSet::<String>::default();
            for BranchToWithArgs { block: succ, .. } in block.successors(context) {
                let succ_livein = &result[&succ];
                cur_live.extend(succ_livein.iter().cloned());
            }
            // Scan the instructions, in reverse.
            for inst in block.instruction_iter(context).rev() {
                match context.values[inst.0].value {
                    ValueDatum::Instruction(Instruction {
                        op: InstOp::Load(ptr),
                        ..
                    }) => {
                        let local_var = get_validate_local_var(context, function, &ptr);
                        match local_var {
                            Some((local, ..)) if locals.contains(&local) => {
                                cur_live.insert(local);
                            }
                            _ => {}
                        }
                    }
                    ValueDatum::Instruction(Instruction {
                        op: InstOp::Store { dst_val_ptr, .. },
                        ..
                    }) => {
                        let local_var = get_validate_local_var(context, function, &dst_val_ptr);
                        match local_var {
                            Some((local, _)) if locals.contains(&local) => {
                                cur_live.remove(&local);
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            if result[block] != cur_live {
                // Whatever's live now, is the live-in for the block.
                result.get_mut(block).unwrap().extend(cur_live);
                changed = true;
            }
        }
    }
    result
}

/// Promote loads of globals constants to SSA registers
/// We promote only non-mutable globals of copy types
fn promote_globals(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut replacements = FxHashMap::<Value, Constant>::default();
    for (_, inst) in function.instruction_iter(context) {
        if let ValueDatum::Instruction(Instruction {
            op: InstOp::Load(ptr),
            ..
        }) = context.values[inst.0].value
        {
            if let ValueDatum::Instruction(Instruction {
                op: InstOp::GetGlobal(global_var),
                ..
            }) = context.values[ptr.0].value
            {
                if !global_var.is_mutable(context)
                    && is_promotable_type(context, global_var.get_inner_type(context))
                {
                    let constant = *global_var
                        .get_initializer(context)
                        .expect("`global_var` is not mutable so it must be initialized");
                    replacements.insert(inst, constant);
                }
            }
        }
    }

    if replacements.is_empty() {
        return Ok(false);
    }

    let replacements = replacements
        .into_iter()
        .map(|(k, v)| (k, Value::new_constant(context, v)))
        .collect::<FxHashMap<_, _>>();

    function.replace_values(context, &replacements, None);

    Ok(true)
}

/// Promote memory values that are accessed via load/store to SSA registers.
pub fn promote_to_registers(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    modified |= promote_globals(context, &function)?;
    modified |= promote_locals(context, analyses, function)?;
    Ok(modified)
}

/// Promote locals to registers. We promote only locals of copy types,
/// whose every use is in a `get_local` without offsets, and the result of
/// such a `get_local` is used only in a load or a store.
pub fn promote_locals(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let safe_locals = filter_usable_locals(context, &function);
    if safe_locals.is_empty() {
        return Ok(false);
    }

    let po: &PostOrder = analyses.get_analysis_result(function);
    let dom_tree: &DomTree = analyses.get_analysis_result(function);
    let dom_fronts: &DomFronts = analyses.get_analysis_result(function);
    let liveins = compute_livein(context, &function, po, &safe_locals);

    // A list of the PHIs we insert in this transform.
    let mut new_phi_tracker = HashSet::<(String, Block)>::new();
    // A map from newly inserted block args to the Local that it's a PHI for.
    let mut worklist = Vec::<(String, Type, Block)>::new();
    let mut phi_to_local = FxHashMap::<Value, String>::default();
    // Insert PHIs for each definition (store) at its dominance frontiers.
    // Start by adding the existing definitions (stores) to a worklist,
    // in program order (reverse post order). This is for faster convergence (or maybe not).
    for (block, inst) in po
        .po_to_block
        .iter()
        .rev()
        .flat_map(|b| b.instruction_iter(context).map(|i| (*b, i)))
    {
        if let ValueDatum::Instruction(Instruction {
            op: InstOp::Store { dst_val_ptr, .. },
            ..
        }) = context.values[inst.0].value
        {
            match get_validate_local_var(context, &function, &dst_val_ptr) {
                Some((local, var)) if safe_locals.contains(&local) => {
                    worklist.push((local, var.get_inner_type(context), block));
                }
                _ => (),
            }
        }
    }
    // Transitively add PHIs, till nothing more to do.
    while let Some((local, ty, known_def)) = worklist.pop() {
        for df in dom_fronts[&known_def].iter() {
            if !new_phi_tracker.contains(&(local.clone(), *df)) && liveins[df].contains(&local) {
                // Insert PHI for this local at block df.
                let index = df.new_arg(context, ty);
                phi_to_local.insert(df.get_arg(context, index).unwrap(), local.clone());
                new_phi_tracker.insert((local.clone(), *df));
                // Add df to the worklist.
                worklist.push((local.clone(), ty, *df));
            }
        }
    }

    // We're just left with rewriting the loads and stores into SSA.
    // For efficiency, we first collect the rewrites
    // and then apply them all together in the next step.
    #[allow(clippy::too_many_arguments)]
    fn record_rewrites(
        context: &mut Context,
        function: &Function,
        dom_tree: &DomTree,
        node: Block,
        safe_locals: &HashSet<String>,
        phi_to_local: &FxHashMap<Value, String>,
        name_stack: &mut MappedStack<String, Value>,
        rewrites: &mut FxHashMap<Value, Value>,
        deletes: &mut Vec<(Block, Value)>,
    ) {
        // Whatever new definitions we find in this block, they must be popped
        // when we're done. So let's keep track of that locally as a count.
        let mut num_local_pushes = IndexMap::<String, u32>::new();

        // Start with relevant block args, they are new definitions.
        for arg in node.arg_iter(context) {
            if let Some(local) = phi_to_local.get(arg) {
                name_stack.push(local.clone(), *arg);
                num_local_pushes
                    .entry(local.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        for inst in node.instruction_iter(context) {
            match context.values[inst.0].value {
                ValueDatum::Instruction(Instruction {
                    op: InstOp::Load(ptr),
                    ..
                }) => {
                    let local_var = get_validate_local_var(context, function, &ptr);
                    match local_var {
                        Some((local, var)) if safe_locals.contains(&local) => {
                            // We should replace all uses of inst with new_stack[local].
                            let new_val = match name_stack.get(&local) {
                                Some(val) => *val,
                                None => {
                                    // Nothing on the stack, let's attempt to get the initializer
                                    let constant = *var
                                        .get_initializer(context)
                                        .expect("We're dealing with an uninitialized value");
                                    Value::new_constant(context, constant)
                                }
                            };
                            rewrites.insert(inst, new_val);
                            deletes.push((node, inst));
                        }
                        _ => (),
                    }
                }
                ValueDatum::Instruction(Instruction {
                    op:
                        InstOp::Store {
                            dst_val_ptr,
                            stored_val,
                        },
                    ..
                }) => {
                    let local_var = get_validate_local_var(context, function, &dst_val_ptr);
                    match local_var {
                        Some((local, _)) if safe_locals.contains(&local) => {
                            // Henceforth, everything that's dominated by this inst must use stored_val
                            // instead of loading from dst_val.
                            name_stack.push(local.clone(), stored_val);
                            num_local_pushes
                                .entry(local)
                                .and_modify(|count| *count += 1)
                                .or_insert(1);
                            deletes.push((node, inst));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        // Update arguments to successor blocks (i.e., PHI args).
        for BranchToWithArgs { block: succ, .. } in node.successors(context) {
            let args: Vec<_> = succ.arg_iter(context).copied().collect();
            // For every arg of succ, if it's in phi_to_local,
            // we pass, as arg, the top value of local
            for arg in args {
                if let Some(local) = phi_to_local.get(&arg) {
                    let ptr = function.get_local_var(context, local).unwrap();
                    let new_val = match name_stack.get(local) {
                        Some(val) => *val,
                        None => {
                            // Nothing on the stack, let's attempt to get the initializer
                            let constant = *ptr
                                .get_initializer(context)
                                .expect("We're dealing with an uninitialized value");
                            Value::new_constant(context, constant)
                        }
                    };
                    let params = node.get_succ_params_mut(context, &succ).unwrap();
                    params.push(new_val);
                }
            }
        }

        // Process dominator children.
        for child in dom_tree.children(node) {
            record_rewrites(
                context,
                function,
                dom_tree,
                child,
                safe_locals,
                phi_to_local,
                name_stack,
                rewrites,
                deletes,
            );
        }

        // Pop from the names stack.
        for (local, pushes) in num_local_pushes.iter() {
            for _ in 0..*pushes {
                name_stack.pop(local);
            }
        }
    }

    let mut name_stack = MappedStack::<String, Value>::default();
    let mut value_replacement = FxHashMap::<Value, Value>::default();
    let mut delete_insts = Vec::<(Block, Value)>::new();
    record_rewrites(
        context,
        &function,
        dom_tree,
        function.get_entry_block(context),
        &safe_locals,
        &phi_to_local,
        &mut name_stack,
        &mut value_replacement,
        &mut delete_insts,
    );

    // Apply the rewrites.
    function.replace_values(context, &value_replacement, None);
    // Delete the loads and stores.
    for (block, inst) in delete_insts {
        block.remove_instruction(context, inst);
    }

    Ok(true)
}
