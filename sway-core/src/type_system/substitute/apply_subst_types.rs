// use crate::{
//     language::ty::TyFunctionParameter, semantic_analysis::TypeCheckContext,
//     type_system::priv_prelude::*,
// };

// pub(crate) trait ApplySubstTypes
// where
//     Self: SubstTypes,
// {
//     fn apply_subst(self, ctx: &TypeCheckContext) -> Substituted<Self> {
//         self.subst(ctx.engines(), &ctx.namespace.subst_list_stack_top())
//     }
// }

// impl<T> ApplySubstTypes for Vec<T>
// where
//     T: ApplySubstTypes,
// {
//     fn apply_subst(self, ctx: &TypeCheckContext) -> Substituted<Self> {
//         self.into_iter().map(|elem| elem.apply_subst(ctx)).collect()
//     }
// }

// impl ApplySubstTypes for TypeId {}

// impl ApplySubstTypes for TyFunctionParameter {}
