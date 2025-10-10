use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::{Declaration, VariableDeclaration},
        ty::{self, TyLetBinding, TyStatement, TyVariableDecl},
    },
    semantic_analysis::{
        symbol_collection_context::SymbolCollectionContext, TypeCheckContext,
    },
    type_system::*,
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Spanned;

impl TyLetBinding {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<VariableDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        TyVariableDecl::collect(handler, engines, ctx, decl_id)
    }

    pub(crate) fn type_check(
        handler: &Handler,
        ctx: &mut TypeCheckContext,
        var_decl: VariableDeclaration,
    ) -> Result<(TyStatement, ty::TyDecl), ErrorEmitted> {
        let span = var_decl.name.span();
        let name = var_decl.name.clone();
        let typed_var_decl = TyVariableDecl::type_check(handler, ctx.by_ref(), var_decl)?;
        let ty_decl = ty::TyDecl::VariableDecl(Box::new(typed_var_decl.clone()));
        ctx.insert_symbol(handler, name, ty_decl.clone())?;
        let statement = TyStatement::Let(typed_var_decl.into());
        Ok((statement, ty_decl))
    }
}

pub(crate) fn parsed_statement_from_decl(
    engines: &Engines,
    decl_id: &ParsedDeclId<VariableDeclaration>,
) -> VariableDeclaration {
    engines.pe().get_variable(decl_id).as_ref().clone()
}

pub(crate) fn declaration_from_statement(statement: &TyStatement) -> Option<ty::TyVariableDecl> {
    match statement {
        TyStatement::Let(binding) => Some(binding.to_variable_decl()),
    }
}
