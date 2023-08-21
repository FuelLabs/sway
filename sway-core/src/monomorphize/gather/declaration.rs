use sway_error::handler::{ErrorEmitted, Handler};

use crate::{decl_engine::DeclId, language::ty, monomorphize::priv_prelude::*, SubstList};

pub(crate) fn gather_from_decl(
    ctx: GatherContext,
    handler: &Handler,
    decl: &ty::TyDecl,
) -> Result<(), ErrorEmitted> {
    match decl {
        ty::TyDecl::VariableDecl(decl) => {
            gather_from_exp(ctx, handler, &decl.body)?;
        }
        ty::TyDecl::ConstantDecl(_) => todo!(),
        ty::TyDecl::FunctionDecl(ty::FunctionDecl {
            decl_id,
            subst_list,
            ..
        }) => {
            gather_from_fn_decl(ctx, handler, decl_id, subst_list.inner())?;
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

fn gather_from_fn_decl(
    mut ctx: GatherContext,
    handler: &Handler,
    decl_id: &DeclId<ty::TyFunctionDecl>,
    subst_list: &SubstList,
) -> Result<(), ErrorEmitted> {
    let decl = ctx.engines.de().get_function(decl_id);

    if !subst_list.is_empty() {
        unimplemented!("{}", decl.name);
    }

    let ty::TyFunctionDecl {
        body,
        parameters,
        return_type,
        ..
    } = decl;

    parameters.iter().for_each(|param| {
        ctx.add_constraint(param.type_argument.type_id.into());
    });
    ctx.add_constraint(return_type.type_id.into());
    gather_from_code_block(ctx.by_ref(), handler, &body)?;

    Ok(())
}
