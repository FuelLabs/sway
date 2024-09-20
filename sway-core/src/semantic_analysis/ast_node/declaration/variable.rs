use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::*,
        ty::{self, TyExpression, TyVariableDecl},
    },
    namespace::ResolvedDeclaration,
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    type_system::*,
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Spanned;
use symbol_collection_context::SymbolCollectionContext;

impl ty::TyVariableDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<VariableDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let var_decl = engines.pe().get_variable(decl_id);
        ctx.insert_parsed_symbol(
            handler,
            engines,
            var_decl.name.clone(),
            Declaration::VariableDeclaration(*decl_id),
        )?;
        TyExpression::collect(handler, engines, ctx, &var_decl.body)
    }

    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        var_decl: VariableDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let engines = &ctx.engines();
        let type_engine = engines.te();

        let mut type_ascription = var_decl.type_ascription.clone();

        type_ascription.type_id = ctx
            .resolve_type(
                handler,
                type_ascription.type_id,
                &type_ascription.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
        let mut ctx = ctx
            .with_type_annotation(type_ascription.type_id)
            .with_help_text(
                "Variable declaration's type annotation does not match up \
                        with the assigned expression's type.",
            );
        let result = ty::TyExpression::type_check(handler, ctx.by_ref(), &var_decl.body);
        let body = result
            .unwrap_or_else(|err| ty::TyExpression::error(err, var_decl.name.span(), engines));

        // TODO: Integers shouldn't be anything special. RHS expressions should be written in
        //       a way to always use the context provided from the LHS, and if the LHS is
        //       an integer, RHS should properly unify or type check should fail.
        //       Remove this special case as a part of the initiative of improving type inference.
        // Integers are special in the sense that we can't only rely on the type of `body`
        // to get the type of the variable. The type of the variable *has* to follow
        // `type_ascription` if `type_ascription` is a concrete integer type that does not
        // conflict with the type of `body` (i.e. passes the type checking above).
        let return_type = match &*type_engine.get(type_ascription.type_id) {
            TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
            _ => match &*type_engine.get(body.return_type) {
                // If RHS type check ends up in an error we want to use the
                // provided type ascription as the variable type. E.g.:
                //   let v: Struct<u8> = Struct<u64> { x: 0 }; // `v` should be "Struct<u8>".
                //   let v: ExistingType = non_existing_identifier; // `v` should be "ExistingType".
                //   let v = <some error>; // `v` will remain "{unknown}".
                // TODO: Refine and improve this further. E.g.,
                //   let v: Struct { /* MISSING FIELDS */ }; // Despite the error, `v` should be of type "Struct".
                TypeInfo::ErrorRecovery(_) => type_ascription.type_id,
                _ => body.return_type,
            },
        };

        if !ctx.code_block_first_pass() {
            let previous_symbol = ctx
                .namespace()
                .module(engines)
                .current_items()
                .check_symbols_unique_while_collecting_unifications(&var_decl.name.clone())
                .ok();

            if let Some(ResolvedDeclaration::Typed(ty::TyDecl::VariableDecl(variable_decl))) =
                previous_symbol
            {
                type_engine.unify(
                    handler,
                    engines,
                    body.return_type,
                    variable_decl.body.return_type,
                    &variable_decl.span(),
                    "",
                    None,
                );
            }
        }

        let typed_var_decl = ty::TyVariableDecl {
            name: var_decl.name.clone(),
            body,
            mutability: ty::VariableMutability::new_from_ref_mut(false, var_decl.is_mutable),
            return_type,
            type_ascription,
        };

        Ok(typed_var_decl)
    }
}

impl TypeCheckAnalysis for TyVariableDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.body.type_check_analyze(handler, ctx)?;
        Ok(())
    }
}

impl TypeCheckFinalization for TyVariableDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        self.body.type_check_finalize(handler, ctx)
    }
}
