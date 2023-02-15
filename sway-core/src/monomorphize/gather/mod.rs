pub(crate) mod code_block;
pub(crate) mod context;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;

use std::sync::RwLock;

use hashbrown::HashMap;
use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(super) fn gather_constraints(
    engines: Engines<'_>,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<impl IntoIterator<Item = Constraint>, ErrorEmitted> {
    let root_namespace = GatherNamespace::init_root(&module.namespace);
    let constraints = RwLock::new(HashMap::new());
    let ctx = GatherContext::from_root(&root_namespace, engines, &constraints);

    gather_from_root(ctx, handler, module)?;

    Ok(constraints.into_inner().unwrap().into_keys())
}
