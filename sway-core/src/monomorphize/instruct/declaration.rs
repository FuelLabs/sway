use sway_error::handler::{ErrorEmitted, Handler};

use crate::{decl_engine::DeclId, language::ty, monomorphize::priv_prelude::*, SubstList};

pub(crate) fn instruct_decl(
    ctx: InstructContext,
    handler: &Handler,
    decl: &ty::TyDecl,
) -> Result<(), ErrorEmitted> {
    match decl {
        ty::TyDecl::VariableDecl(decl) => {
            instruct_exp(ctx, handler, &decl.body)?;
        }
        ty::TyDecl::ConstantDecl(_) => todo!(),
        ty::TyDecl::FunctionDecl(ty::FunctionDecl {
            decl_id,
            subst_list,
            ..
        }) => {
            instruct_fn_decl(ctx, handler, decl_id, subst_list.inner())?;
        }
        ty::TyDecl::TraitDecl(_) => todo!(),
        ty::TyDecl::StructDecl(_) => todo!(),
        ty::TyDecl::EnumDecl(_) => todo!(),
        ty::TyDecl::EnumVariantDecl(_) => todo!(),
        ty::TyDecl::ImplTrait(_) => todo!(),
        ty::TyDecl::AbiDecl(_) => todo!(),
        ty::TyDecl::GenericTypeForFunctionScope(_) => todo!(),
        ty::TyDecl::StorageDecl(_) => todo!(),
        ty::TyDecl::ErrorRecovery(_, _) => {}
        ty::TyDecl::TypeAliasDecl(_) => todo!(),
        ty::TyDecl::TypeDecl(_) => todo!(),
    }
    Ok(())
}

fn instruct_fn_decl(
    mut ctx: InstructContext,
    handler: &Handler,
    decl_id: &DeclId<ty::TyFunctionDecl>,
    subst_list: &SubstList,
) -> Result<(), ErrorEmitted> {
    let decl = ctx.engines.de().get_function(decl_id);

    if !subst_list.is_empty() {
        unimplemented!("{}", decl.name);
    }

    let ty::TyFunctionDecl { body, .. } = decl;

    // NOTE: todo here
    instruct_code_block(ctx.by_ref(), handler, &body)?;

    Ok(())
}
