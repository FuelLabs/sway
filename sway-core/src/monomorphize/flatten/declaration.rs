use crate::{
    decl_engine::DeclId, language::ty::*, monomorphize::priv_prelude::*, Engines, SubstList,
};

pub(crate) fn flatten_decl(engines: Engines<'_>, decl: TyDecl) -> TyDecl {
    match decl {
        ty::TyDecl::VariableDecl(decl) => {
            flatten_exp(engines, &decl.body);
        }
        ty::TyDecl::ConstantDecl { .. } => todo!(),
        ty::TyDecl::FunctionDecl {
            decl_id,
            subst_list,
            ..
        } => {
            flatten_fn_decl(engines, decl_id, subst_list.inner());
        }
        ty::TyDecl::TraitDecl { .. } => todo!(),
        ty::TyDecl::StructDecl { .. } => todo!(),
        ty::TyDecl::EnumDecl { .. } => todo!(),
        ty::TyDecl::ImplTrait { .. } => todo!(),
        ty::TyDecl::AbiDecl { .. } => todo!(),
        ty::TyDecl::GenericTypeForFunctionScope { .. } => todo!(),
        ty::TyDecl::StorageDecl { .. } => todo!(),
        ty::TyDecl::ErrorRecovery(_) => {}
        ty::TyDecl::TypeAliasDecl { .. } => todo!(),
    }
}

fn flatten_fn_decl(
    engines: Engines<'_>,
    decl_id: &DeclId<ty::TyFunctionDecl>,
    subst_list: &SubstList,
) {
    let decl = engines.de().get_function(decl_id);

    if !subst_list.is_empty() {
        unimplemented!("{}", decl.name);
    }

    let ty::TyFunctionDecl { body, .. } = decl;

    // NOTE: todo here
    flatten_code_block(engines, &body);
}
