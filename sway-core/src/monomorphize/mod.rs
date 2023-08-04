#![allow(dead_code)]

mod constraint;
mod gather;
mod instruct;
mod instructions;
mod priv_prelude;
mod solve;

use crate::{engine_threading::*, language::ty};

use priv_prelude::*;
use sway_error::handler::{ErrorEmitted, Handler};

pub(super) fn monomorphize(
    handler: &Handler,
    engines: &Engines,
    module: &mut ty::TyModule,
) -> Result<(), ErrorEmitted> {
    // Gather the constraints from the typed AST.
    let constraints = gather_constraints(engines, handler, module)?;

    // Solve the constraints and get back instructions from the solver.
    let mut solver = Solver::new(engines);
    solver.solve(handler, constraints)?;
    let instructions = solver.into_instructions();

    // Use the new instructions to monomorphize the AST.
    apply_instructions(engines, handler, instructions, module)?;

    Ok(())
}
