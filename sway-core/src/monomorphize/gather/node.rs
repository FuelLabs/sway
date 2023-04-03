use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_node(ctx: GatherContext, node: &ty::TyAstNodeContent) {
    match node {
        ty::TyAstNodeContent::Declaration(decl) => {
            gather_from_decl(ctx, decl);
        }
        ty::TyAstNodeContent::Expression(exp) => {
            gather_from_exp(ctx, exp);
        }
        ty::TyAstNodeContent::ImplicitReturnExpression(exp) => {
            gather_from_exp(ctx, exp);
        }
        ty::TyAstNodeContent::SideEffect(_) => {}
    }
}
