use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

pub(crate) fn find_from_decl<'a>(engines: Engines<'_>, decl: &'a TyDecl) -> Findings<'a> {
    use TyDecl::*;
    match decl {
        VariableDecl(decl) => find_from_exp(engines, &decl.body),
        FunctionDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => Findings::from_fn_decl(decl_id, subst_list.inner()),
        TraitDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => Findings::from_trait_decl(decl_id, subst_list.inner()),
        StructDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => Findings::from_struct_decl(decl_id, subst_list.inner()),
        EnumDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => Findings::from_enum_decl(decl_id, subst_list.inner()),
        ImplTrait {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => Findings::from_impl_trait(decl_id, subst_list.inner()),
        AbiDecl { .. } => todo!(),
        StorageDecl { .. } => todo!(),
        TypeAliasDecl { .. } => todo!(),
        ConstantDecl { .. }
        | ErrorRecovery(_)
        | GenericTypeForFunctionScope { .. }
        | EnumVariantDecl { .. } => Findings::new(),
    }
}
