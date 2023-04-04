//! This module gathers a list of [Constraint]s from a typed AST.

pub(crate) mod code_block;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod findings;
pub(crate) mod module;
pub(crate) mod node;

use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

/// Gathers [Constraint]s from a typed AST.
pub(super) fn flatten_ast(engines: Engines<'_>, module: TyModule) -> (TyModule, StateGraphs) {
    let findings = find_from_root(engines, &module);
    todo!()
}
