//! This module gathers a list of [Constraint]s from a typed AST.

pub(crate) mod code_block;
pub(crate) mod context;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;
pub(crate) mod type_system;
pub(crate) mod collect_from;

use std::sync::RwLock;

use hashbrown::HashMap;

use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

/// Gathers [Constraint]s from a typed AST.
pub(super) fn gather_constraints(
    engines: Engines<'_>,
    module: &TyModule,
) -> impl IntoIterator<Item = Constraint> {
    let constraints = RwLock::new(HashMap::new());
    let ctx = GatherContext::new(engines, &constraints);

    gather_from_root(ctx, module);

    constraints.into_inner().unwrap().into_keys()
}
