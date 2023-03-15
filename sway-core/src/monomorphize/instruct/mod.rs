//! This module applies a list of [Instruction]s to a typed AST to
//! monomorphize it.

pub(crate) mod code_block;
pub(crate) mod context;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;

use std::sync::RwLock;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

/// Uses [Instruction]s to monomorphize a typed AST.
pub(crate) fn apply_instructions(
    engines: Engines<'_>,
    handler: &Handler,
    instructions: Vec<Instruction>,
    module: &mut ty::TyModule,
) -> Result<(), ErrorEmitted> {
    let instructions = RwLock::new(InstructionItems::new(instructions));
    let ctx = InstructContext::from_root(engines, &instructions);

    instruct_root(ctx, handler, module)?;

    Ok(())
}
