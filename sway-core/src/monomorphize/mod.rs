#![allow(dead_code)]

mod constraint;
// mod flatten;
mod gather;
mod instruct;
mod instructions;
mod priv_prelude;
mod solve;

use crate::{engine_threading::*, language::ty::*, CompileResult};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, mut module: TyModule) -> CompileResult<TyModule> {
    CompileResult::with_handler(|h| {
        // // Flatten and preprocess the typed AST.
        // let module = flatten_ast(engines, module);

        // Gather the constraints from the typed AST.
        let constraints = gather_constraints(engines, &module);

        // Solve the constraints and get back instructions from the solver.
        let mut solver = Solver::new(engines);
        solver.solve(h, constraints)?;
        let instructions = solver.into_instructions();

        // Use the new instructions to monomorphize the AST.
        apply_instructions(engines, h, instructions, &mut module)?;

        Ok(module)
    })
}
