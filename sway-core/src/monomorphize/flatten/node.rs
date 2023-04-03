use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

pub(crate) fn flatten_node(engines: Engines<'_>, node: &mut TyAstNodeContent) -> TyAstNodeContent {
    use TyAstNodeContent::*;
    match node {
        Declaration(decl) => flatten_decl(engines, decl),
        Expression(exp) => flatten_exp(engines, exp),
        ImplicitReturnExpression(exp) => flatten_exp(engines, exp),
        SideEffect(_) => {}
    }
}
