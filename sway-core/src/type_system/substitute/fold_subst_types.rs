// use crate::{decl_engine::*, type_system::priv_prelude::*, Engines};

// pub(crate) trait FoldSubstTypes {
//     type Output;

//     fn fold_subst(&self, engines: Engines<'_>) -> Substituted<Self::Output>;
// }

// impl<T> FoldSubstTypes for DeclRef<DeclId<T>>
// where
//     T: SubstTypes,
//     DeclEngine: DeclEngineGet<DeclId<T>, T>,
// {
//     type Output = T;

//     fn fold_subst(&self, engines: Engines<'_>) -> Substituted<T> {
//         let decl_engine = engines.de();
//         self.fold(|decl_id, subst_list| {
//             let v = decl_engine.get(decl_id);
//             v.subst(engines, subst_list)
//         })
//     }
// }
