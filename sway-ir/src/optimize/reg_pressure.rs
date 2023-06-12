//! A pass to reduce register pressure.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    get_loaded_symbols, get_stored_symbols, AnalysisResults, Context, EscapedSymbols, Function,
    IrError, Pass, PassMutability, ScopedPass, Symbol, Value, ESCAPED_SYMBOLS_NAME,
};

pub const REG_PRESSURE_OPT_NAME: &str = "regpressure";

pub fn create_reg_pressure_pass() -> Pass {
    Pass {
        name: REG_PRESSURE_OPT_NAME,
        descr: "Register pressure reduction.",
        deps: vec![ESCAPED_SYMBOLS_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(reg_pressure_opt)),
    }
}

pub fn reg_pressure_opt(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;

    let escaped_symbols: &EscapedSymbols = analyses.get_analysis_result(function);

    'block_opt: for block in function.block_iter(context) {
        // Since we don't have direct info on which block an instruction is in.
        let insts_in_block: FxHashSet<_> = block.instruction_iter(context).collect();
        // The key must be lexically before its values in this map.
        let mut key_before_vals: FxHashMap<Value, FxHashSet<Value>> = FxHashMap::default();
        // The key must be lexically after its values in this map.
        let mut key_after_vals: FxHashMap<Value, FxHashSet<Value>> = FxHashMap::default();
        // In our reverse traversal, what's the last store of a symbol that we saw?
        let mut last_seen_store: FxHashMap<Symbol, Value> = FxHashMap::default();
        // In our reverse traversal, what're the loads of a symbol we've seen so far.
        let mut sym_loads: FxHashMap<Symbol, FxHashSet<Value>> = FxHashMap::default();

        // Collect ordering dependences in the block.
        // NOTE: We're reverse traversing here.
        let block_size = block.num_instructions(context) as u32;
        for val in block.instruction_iter(context).rev() {
            // A utility function.
            let mut set_order = |before, after| {
                key_before_vals
                    .entry(before)
                    .and_modify(|afters| {
                        afters.insert(after);
                    })
                    .or_insert([after].into_iter().collect());
                key_after_vals
                    .entry(after)
                    .and_modify(|befores| {
                        befores.insert(before);
                    })
                    .or_insert([before].into_iter().collect());
            };

            let inst = val.get_instruction(context).unwrap();
            // each operand must be scheduled before inst.
            for operand in inst.get_operands() {
                if insts_in_block.contains(&operand) {
                    set_order(operand, val);
                }
            }

            // for every symbol that this stores to, inst must be scheduled before
            // any already seen stores or loads of that symbol.
            for sym in get_stored_symbols(context, val) {
                if escaped_symbols.contains(&sym) {
                    continue 'block_opt;
                }
                if let Some(store_of_sym) = last_seen_store.get(&sym) {
                    set_order(val, *store_of_sym);
                }
                for load_of_sym in sym_loads.get(&sym).unwrap_or(&FxHashSet::default()) {
                    set_order(val, *load_of_sym);
                }
                last_seen_store.insert(sym, val);
            }

            // for every symbol that this loads from, inst must be scheduled before
            // any already seen store of that symbol.
            for sym in get_loaded_symbols(context, val) {
                if escaped_symbols.contains(&sym) {
                    continue 'block_opt;
                }
                if let Some(store_of_sym) = last_seen_store.get(&sym) {
                    set_order(val, *store_of_sym);
                }
                sym_loads
                    .entry(sym)
                    .and_modify(|loads| {
                        loads.insert(val);
                    })
                    .or_insert([val].into_iter().collect());
            }
        }

        let mut new_block_insts_rev = vec![];
        // A map of live variables (as we schedule instructions from the block end)
        // and the position (terminator is at position `block_size`) at which they became live.
        let mut live_till = FxHashMap::<Value, u32>::default();
        let (mut terminator, ready): (Vec<_>, Vec<_>) = block
            .instruction_iter(context)
            .filter(|inst| !key_before_vals.contains_key(inst))
            .partition(|inst| inst.is_terminator(context));
        let mut ready: FxHashSet<Value> = ready.into_iter().collect();
        let terminator = terminator.remove(0);

        // The ones that are already ready are those that have uses beyond
        // this block (assuming dce has run). Let's schedule them with priority,
        // or we'll end up putting them to the start of the block.
        for &read in ready.iter().filter(|inst| {
            !inst
                .get_instruction(context)
                .unwrap()
                .may_have_side_effect()
        }) {
            live_till.insert(read, block_size + 1);
        }

        // Schedule an instruction as-per the new order.
        fn schedule(
            ready: &mut FxHashSet<Value>,
            new_block_insts_rev: &mut Vec<Value>,
            live_till: &mut FxHashMap<Value, u32>,
            key_before_vals: &mut FxHashMap<Value, FxHashSet<Value>>,
            key_after_vals: &mut FxHashMap<Value, FxHashSet<Value>>,
            inst: Value,
            counter: u32,
        ) {
            ready.remove(&inst);
            new_block_insts_rev.push(inst);
            for &operand in key_after_vals.get(&inst).unwrap_or(&FxHashSet::default()) {
                live_till.entry(operand).or_insert(counter);
                if let Some(afters) = key_before_vals.get_mut(&operand) {
                    afters.remove(&inst);
                    if afters.is_empty() {
                        ready.insert(operand);
                    }
                }
            }
        }
        // Chose the next instruction to schedule based on heuristics.
        fn choose(ready: &FxHashSet<Value>, live_till: &FxHashMap<Value, u32>) -> Option<Value> {
            ready
                .iter()
                .max_by(|&inst1, &inst2| {
                    let inst1_live_till = live_till.get(inst1).cloned().unwrap_or_default();
                    let inst2_live_till = live_till.get(inst2).cloned().unwrap_or_default();
                    inst1_live_till.cmp(&inst2_live_till)
                })
                .cloned()
        }

        // Let's schedule the terminator first.
        schedule(
            &mut ready,
            &mut new_block_insts_rev,
            &mut live_till,
            &mut key_before_vals,
            &mut key_after_vals,
            terminator,
            block_size,
        );

        let mut next_pos = block_size - 1;
        while !ready.is_empty() {
            let next = choose(&ready, &live_till).unwrap();
            schedule(
                &mut ready,
                &mut new_block_insts_rev,
                &mut live_till,
                &mut key_before_vals,
                &mut key_after_vals,
                next,
                next_pos,
            );
            assert!(
                next_pos > 0,
                "Bug in scheduler. Ready wasn't empty, so we can't have reached 0 already"
            );
            next_pos -= 1;
        }

        assert!(
            next_pos == 0 && new_block_insts_rev.len() == block_size as usize,
            "Something went wrong, we didn't schedule all instructions"
        );

        context.blocks[block.0].instructions = new_block_insts_rev.into_iter().rev().collect();

        modified = true;
    }

    Ok(modified)
}
