#![allow(dead_code)]

mod constraint;
mod gather;
mod instruct;
mod instructions;
mod priv_prelude;
mod solve;

use crate::{engine_threading::*, language::ty, CompileResult};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: &mut ty::TyModule) -> CompileResult<()> {
    CompileResult::with_handler(|h| {
        // Gather the constraints from the typed AST.
        let constraints = gather_constraints(engines, h, module)?;

        // Solve the constraints and get back instructions from the solver.
        let mut solver = Solver::new(engines);
        solver.solve(h, constraints)?;
        let instructions = solver.into_instructions();

        // Use the new instructions to monomorphize the AST.
        apply_instructions(engines, h, instructions, module)?;

        Ok(())
    })
}
