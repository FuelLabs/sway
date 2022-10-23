/// Promote local memory to SSA registers.
/// This pass is essentially SSA construction. A good readable reference is:
/// https://www.cs.princeton.edu/~appel/modern/c/
/// We use block arguments instead of explicit PHI nodes. Conceptually,
/// they are both the same.
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet};
use sway_utils::mapped_stack::MappedStack;

use crate::{
    compute_dom_fronts, dominator::compute_dom_tree, Block, BranchToWithArgs, Context, DomTree,
    Function, Instruction, IrError, Pointer, PostOrder, Type, Value, ValueDatum,
};

// Check if a value is a valid (for our optimization) local pointer
fn get_validate_local_pointer(
    context: &Context,
    function: &Function,
    val: &Value,
) -> Option<(String, Pointer, bool)> {
    match context.values[val.0].value {
        ValueDatum::Instruction(Instruction::GetPointer {
            base_ptr,
            ptr_ty,
            offset,
        }) => {
            let is_valid = ptr_ty
                .get_type(context)
                .eq(context, base_ptr.get_type(context))
                && offset == 0;
            let name = function.lookup_local_name(context, &base_ptr);
            name.map(|name| (name.clone(), base_ptr, is_valid))
        }
        _ => None,
    }
}

// Returns those locals that can be promoted to SSA registers.
fn filter_usable_locals(context: &mut Context, function: &Function) -> HashSet<String> {
    let mut locals: HashSet<String> = function
        .locals_iter(context)
        .filter(|(_, ptr)| (**ptr).get_type(context).is_copy_type())
        .map(|(name, _)| name.clone())
        .collect();

    for (_, inst) in function.instruction_iter(context) {
        if let Some((local, _, valid)) = get_validate_local_pointer(context, function, &inst) {
            if !valid {
                locals.remove(&local);
            }
        }
    }
    locals
}

// For each block, compute the set of locals that are live-in.
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
                    ValueDatum::Instruction(Instruction::Load(ptr)) => {
                        let local_ptr = get_validate_local_pointer(context, function, &ptr);
                        match local_ptr {
                            Some((local, ..)) if locals.contains(&local) => {
                                cur_live.insert(local);
                            }
                            _ => {}
                        }
                    }
                    ValueDatum::Instruction(Instruction::Store { dst_val, .. }) => {
                        let local_ptr = get_validate_local_pointer(context, function, &dst_val);
                        match local_ptr {
                            Some((local, _, _)) if locals.contains(&local) => {
                                cur_live.remove(&local);
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            // Whatever's live now, is the live-in for the block.
            result.get_mut(block).unwrap().extend(cur_live);
        }
    }
    result
}

/// Promote local values that are accessed via load/store to SSA registers.
/// We promote only locals of non-copy type, whose every use is in a `get_ptr`
/// without offsets, and the result of such a `get_ptr` is used only in a load
/// or a store.
pub fn promote_to_registers(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let safe_locals = filter_usable_locals(context, function);

    if safe_locals.is_empty() {
        return Ok(false);
    }

    let (dom_tree, po) = compute_dom_tree(context, function);
    let dom_fronts = compute_dom_fronts(context, function, &dom_tree);
    // print!(
    //     "{}\n{}\n{}",
    //     function.dot_cfg(context),
    //     print_dot(context, function.get_name(context), &dom_tree),
    //     print_dom_fronts(context, function.get_name(context), &dom_fronts),
    // );
    let liveins = compute_livein(context, function, &po, &safe_locals);

    // A list of the PHIs we insert in this transform.
    let mut new_phi_tracker = HashSet::<(String, Block)>::new();
    // A map from newly inserted block args to the Local that it's a PHI for.
    let mut worklist = Vec::<(String, Type, Block)>::new();
    let mut phi_to_local = FxHashMap::<Value, String>::default();
    // Insert PHIs for each definition (store) at its dominance frontiers.
    // Start by adding the existing definitions (stores) to a worklist.
    for (block, inst) in function.instruction_iter(context) {
        if let ValueDatum::Instruction(Instruction::Store { dst_val, .. }) =
            context.values[inst.0].value
        {
            match get_validate_local_pointer(context, function, &dst_val) {
                Some((local, ptr, ..)) if safe_locals.contains(&local) => {
                    worklist.push((local, *ptr.get_type(context), block));
                }
                _ => (),
            }
        }
    }
    // Transitively add PHIs, till nothing more to do.
    while !worklist.is_empty() {
        let (local, ty, known_def) = worklist.pop().unwrap();
        for df in dom_fronts[&known_def].iter() {
            if !new_phi_tracker.contains(&(local.clone(), *df)) && liveins[df].contains(&local) {
                // print!(
                //     "Adding PHI for {} in block {}\n",
                //     local,
                //     df.get_label(context)
                // );
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
        let mut num_local_pushes = HashMap::<String, u32>::new();

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
                ValueDatum::Instruction(Instruction::Load(ptr)) => {
                    let local_ptr = get_validate_local_pointer(context, function, &ptr);
                    match local_ptr {
                        Some((local, ptr, _)) if safe_locals.contains(&local) => {
                            // We should replace all uses of inst with new_stack[local].
                            let new_val = match name_stack.get(&local) {
                                Some(val) => *val,
                                None => {
                                    // Nothing on the stack, let's attempt to get the initializer
                                    Value::new_constant(
                                        context,
                                        ptr.get_initializer(context)
                                            .expect("We're dealing with an uninitialized value")
                                            .clone(),
                                    )
                                }
                            };
                            rewrites.insert(inst, new_val);
                            deletes.push((node, inst));
                        }
                        _ => (),
                    }
                }
                ValueDatum::Instruction(Instruction::Store {
                    dst_val,
                    stored_val,
                }) => {
                    let local_ptr = get_validate_local_pointer(context, function, &dst_val);
                    match local_ptr {
                        Some((local, _, _)) if safe_locals.contains(&local) => {
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
                    let ptr = function.get_local_ptr(context, local).unwrap();
                    let new_val = match name_stack.get(local) {
                        Some(val) => *val,
                        None => {
                            // Nothing on the stack, let's attempt to get the initializer
                            Value::new_constant(
                                context,
                                ptr.get_initializer(context)
                                    .expect("We're dealing with an uninitialized value")
                                    .clone(),
                            )
                        }
                    };
                    let params = node.get_succ_params_mut(context, &succ).unwrap();
                    params.push(new_val);
                }
            }
        }

        // Process dominator children.
        for child in dom_tree[&node].children.iter() {
            record_rewrites(
                context,
                function,
                dom_tree,
                *child,
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
        function,
        &dom_tree,
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
