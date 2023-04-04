use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(crate) fn find_from_node<'a>(
    engines: Engines<'_>,
    node: &'a ty::TyAstNodeContent,
) -> Findings<'a> {
    match node {
        ty::TyAstNodeContent::Declaration(decl) => find_from_decl(engines, decl),
        ty::TyAstNodeContent::Expression(exp) => find_from_exp(engines, exp),
        ty::TyAstNodeContent::ImplicitReturnExpression(exp) => find_from_exp(engines, exp),
        ty::TyAstNodeContent::SideEffect(_) => Findings::new(),
    }
}
