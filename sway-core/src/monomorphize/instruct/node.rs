use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn instruct_node(
    ctx: InstructContext,
    handler: &Handler,
    node: &ty::TyAstNodeContent,
) -> Result<(), ErrorEmitted> {
    match node {
        ty::TyAstNodeContent::Declaration(decl) => instruct_decl(ctx, handler, decl),
        ty::TyAstNodeContent::Expression(exp) => instruct_exp(ctx, handler, exp),
        ty::TyAstNodeContent::ImplicitReturnExpression(exp) => instruct_exp(ctx, handler, exp),
        ty::TyAstNodeContent::SideEffect(_) => Ok(()),
    }
}
