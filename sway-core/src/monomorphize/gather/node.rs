use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_node(
    ctx: GatherContext,
    handler: &Handler,
    node: &ty::TyAstNodeContent,
) -> Result<(), ErrorEmitted> {
    match node {
        ty::TyAstNodeContent::Declaration(decl) => gather_from_decl(ctx, handler, decl),
        ty::TyAstNodeContent::Expression(exp) => gather_from_exp(ctx, handler, exp),
        ty::TyAstNodeContent::ImplicitReturnExpression(exp) => gather_from_exp(ctx, handler, exp),
        ty::TyAstNodeContent::SideEffect(_) => Ok(()),
    }
}
