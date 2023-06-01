use crate::{decl_engine::DeclId, language::ty::*, monomorphize::priv_prelude::*, type_system::*};

pub(crate) fn gather_from_decl(ctx: GatherContext, decl: &TyDecl) {
    use TyDecl::*;
    match decl {
        VariableDecl(decl) => {
            gather_from_exp(ctx, &decl.body);
        }
        ConstantDecl { .. } => todo!(),
        FunctionDecl {
            decl_id,
            subst_list,
            ..
        } => {
            gather_from_fn_decl(ctx, decl_id, subst_list.inner());
        }
        TraitDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => {
            gather_from_trait_decl(ctx, decl_id, subst_list.inner());
        }
        StructDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => {
            gather_from_struct_decl(ctx, decl_id, subst_list.inner());
        }
        EnumDecl {
            name: _,
            decl_id,
            subst_list,
            decl_span: _,
        } => {
            gather_from_enum_decl(ctx, decl_id, subst_list.inner());
        }
        EnumVariantDecl { .. } => todo!(),
        ImplTrait { .. } => todo!(),
        AbiDecl { .. } => todo!(),
        GenericTypeForFunctionScope { .. } => todo!(),
        StorageDecl { .. } => todo!(),
        ErrorRecovery(_) => {}
        TypeAliasDecl { .. } => todo!(),
    }
}

fn gather_from_fn_decl(
    ctx: GatherContext,
    decl_id: &DeclId<TyFunctionDecl>,
    subst_list: &SubstList,
) {
    ctx.add_constraint(Constraint::mk_fn_decl(decl_id, subst_list));
    let fn_decl = ctx.decl_engine.get_function(decl_id);
    for param in fn_decl.parameters {
        gather_from_ty(ctx, param.type_argument.type_id);
    }
    gather_from_ty(ctx, fn_decl.return_type.type_id);
    gather_from_code_block(ctx, &fn_decl.body);
}

fn gather_from_trait_decl(
    ctx: GatherContext,
    decl_id: &DeclId<TyTraitDecl>,
    subst_list: &SubstList,
) {
    ctx.add_constraint(Constraint::mk_trait_decl(decl_id, subst_list));
    let trait_decl = ctx.decl_engine.get_trait(decl_id);
    todo!();
}

fn gather_from_struct_decl(
    ctx: GatherContext,
    decl_id: &DeclId<TyStructDecl>,
    subst_list: &SubstList,
) {
    ctx.add_constraint(Constraint::mk_struct_decl(decl_id, subst_list));
    let struct_decl = ctx.decl_engine.get_struct(decl_id);
    for field in struct_decl.fields {
        gather_from_ty(ctx, field.type_argument.type_id);
    }
}

fn gather_from_enum_decl(ctx: GatherContext, decl_id: &DeclId<TyEnumDecl>, subst_list: &SubstList) {
    ctx.add_constraint(Constraint::mk_enum_decl(decl_id, subst_list));
    let enum_decl = ctx.decl_engine.get_enum(decl_id);
    for variant in enum_decl.variants {
        gather_from_ty(ctx, variant.type_argument.type_id);
    }
}
