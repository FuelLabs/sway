#![allow(dead_code)]

mod constraint;
mod flatten;
mod gather;
mod instruct;
mod instructions;
mod priv_prelude;
mod solve;
mod state_graphs;
mod collect;
mod state_graph;
mod mono_item;

use std::sync::RwLock;

use crate::{engine_threading::*, language::ty::*, CompileResult};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: TyModule) -> CompileResult<TyModule> {
    CompileResult::with_handler(|h| {
        // Flatten and preprocess the typed AST.
        let (mut module, state_graphs) = flatten_ast(engines, module);
        let state_graphs = RwLock::new(state_graphs);

        // Gather the constraints from the typed AST.
        let constraints = gather_constraints(engines, &module);

        // Solve the constraints and get back instructions from the solver.
        let mut solver = Solver::new(engines, &state_graphs);
        solver.solve(h, constraints)?;
        let instructions = solver.into_instructions();

        // Use the new instructions to monomorphize the AST.
        apply_instructions(engines, h, instructions, &mut module)?;

        Ok(module)
    })
}
