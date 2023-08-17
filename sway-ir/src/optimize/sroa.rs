//! Scalar Replacement of Aggregates

use rustc_hash::FxHashSet;

use crate::{
    compute_escaped_symbols, get_loaded_ptr_values, get_loaded_symbols, get_stored_ptr_values,
    get_stored_symbols, get_symbol, AnalysisResults, Context, Function, IrError, LocalVar, Pass,
    PassMutability, ScopedPass, Symbol,
};

pub const SROA_NAME: &str = "sroa";

pub fn create_sroa_pass() -> Pass {
    Pass {
        name: SROA_NAME,
        descr: "Scalar replacement of aggregates.",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(sroa)),
    }
}

/// Promote aggregates to scalars, so that other optimizations
/// such as mem2reg can treat them as any other SSA value.
pub fn sroa(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    todo!()
}

/// Only the following aggregates can be scalarised:
/// 1. Does not escape.
/// 2. Is always accessed via a scalar (register sized) field.
///    i.e., The entire aggregate or a sub-aggregate isn't loaded / stored.
fn candidate_symbols(context: &Context, function: Function) -> FxHashSet<Symbol> {
    let escaped_symbols = compute_escaped_symbols(context, &function);
    let mut candidates: FxHashSet<Symbol> = function
        .locals_iter(context)
        .filter_map(|(_, l)| {
            let sym = Symbol::Local(*l);
            escaped_symbols.contains(&sym).then(|| sym)
        })
        .collect();

    // We walk the function to remove from `candidates`, any local that is accessed
    // by a bigger-than-register sized load / store.
    for (_, inst) in function.instruction_iter(context) {
        let loaded_pointers = get_loaded_ptr_values(context, inst);
        let stored_pointers = get_stored_ptr_values(context, inst);
        let cannot_handle = loaded_pointers
            .iter()
            .chain(stored_pointers.iter())
            .any(|v| {
                v.match_ptr_type(context).is_some_and(|pointee_ty| {
                    super::target_fuel::is_demotable_type(context, &pointee_ty)
                })
            });
        if cannot_handle {
            for sym in get_loaded_symbols(context, inst)
                .iter()
                .chain(get_stored_symbols(context, inst).iter())
            {
                candidates.remove(sym);
            }
        }
    }

    candidates
}
