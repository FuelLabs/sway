use sway_error::handler::{ErrorEmitted, Handler};

use crate::{decl_engine::DeclId, language::ty, monomorphize::priv_prelude::*, SubstList};

pub(crate) fn instruct_decl(
    ctx: InstructContext,
    handler: &Handler,
    decl: &ty::TyDeclaration,
) -> Result<(), ErrorEmitted> {
    match decl {
        ty::TyDeclaration::VariableDeclaration(decl) => {
            instruct_exp(ctx, handler, &decl.body)?;
        }
        ty::TyDeclaration::ConstantDeclaration { .. } => todo!(),
        ty::TyDeclaration::FunctionDeclaration {
            decl_id,
            subst_list,
            ..
        } => {
            instruct_fn_decl(ctx, handler, decl_id, subst_list.inner())?;
        }
        ty::TyDeclaration::TraitDeclaration { .. } => todo!(),
        ty::TyDeclaration::StructDeclaration { .. } => todo!(),
        ty::TyDeclaration::EnumDeclaration { .. } => todo!(),
        ty::TyDeclaration::ImplTrait { .. } => todo!(),
        ty::TyDeclaration::AbiDeclaration { .. } => todo!(),
        ty::TyDeclaration::GenericTypeForFunctionScope { .. } => todo!(),
        ty::TyDeclaration::StorageDeclaration { .. } => todo!(),
        ty::TyDeclaration::ErrorRecovery(_) => {}
        ty::TyDeclaration::TypeAliasDeclaration { .. } => todo!(),
    }

    Ok(())
}

fn instruct_fn_decl(
    mut ctx: InstructContext,
    handler: &Handler,
    decl_id: &DeclId<ty::TyFunctionDeclaration>,
    subst_list: &SubstList,
) -> Result<(), ErrorEmitted> {
    let decl = ctx.decl_engine.get_function(decl_id);

    if !subst_list.is_empty() {
        unimplemented!("{}", decl.name);
    }

    let ty::TyFunctionDeclaration { body, .. } = decl;

    // NOTE: todo here
    instruct_code_block(ctx.by_ref(), handler, &body)?;

    Ok(())
}
