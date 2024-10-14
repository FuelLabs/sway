use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::{Declaration, TypeAliasDeclaration},
        ty::TyTypeAliasDecl,
    },
    semantic_analysis::{
        symbol_collection_context::SymbolCollectionContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};

impl TyTypeAliasDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<TypeAliasDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let type_alias = engines.pe().get_type_alias(decl_id);
        ctx.insert_parsed_symbol(
            handler,
            engines,
            type_alias.name.clone(),
            Declaration::TypeAliasDeclaration(*decl_id),
        )
    }
}

impl TypeCheckFinalization for TyTypeAliasDecl {
    fn type_check_finalize(
        &mut self,
        _handler: &Handler,
        _ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}
