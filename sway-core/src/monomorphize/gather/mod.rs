//! This module gathers a list of [Constraint]s from a typed AST.

pub(crate) mod code_block;
pub(crate) mod context;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;
pub(crate) mod type_system;

use std::sync::RwLock;

use hashbrown::HashMap;

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

/// Gathers [Constraint]s from a typed AST.
pub(super) fn gather_constraints(
    engines: Engines<'_>,
    module: &ty::TyModule,
) -> impl IntoIterator<Item = Constraint> {
    let root_namespace = GatherNamespace::init_root(&module.namespace);
    let constraints = RwLock::new(HashMap::new());
    let ctx = GatherContext::from_root(&root_namespace, engines, &constraints);

    gather_from_root(ctx, module);

    constraints.into_inner().unwrap().into_keys()
}
