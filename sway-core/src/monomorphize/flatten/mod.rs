//! This flattens and folds the [SubstList]'s in the AST to remove all instances
//! of the [TypeInfo][TypeParam] variant.

pub(crate) mod code_block;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;

use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

/// Flattens and folds the [SubstList]'s in the AST.
pub(crate) fn flatten_ast(engines: Engines<'_>, module: TyModule) -> TyModule {
    flatten_root(engines, module)
}
