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
        ty::TyDecl::ConstantDecl { .. } => todo!(),
        ty::TyDecl::FunctionDecl {
            decl_id,
            subst_list,
            ..
        } => {
            instruct_fn_decl(ctx, handler, decl_id, subst_list.inner())?;
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

    Ok(())
}

fn instruct_fn_decl(
    mut ctx: InstructContext,
    handler: &Handler,
    decl_id: &DeclId<ty::TyFunctionDecl>,
    subst_list: &SubstList,
) -> Result<(), ErrorEmitted> {
    let decl = ctx.decl_engine.get_function(decl_id);

    if !subst_list.is_empty() {
        unimplemented!("{}", decl.name);
    }

    let ty::TyFunctionDecl { body, .. } = decl;

    // NOTE: todo here
    instruct_code_block(ctx.by_ref(), handler, &body)?;

    Ok(())
}
