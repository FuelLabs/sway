use std::sync::Arc;

use crate::{
    language::{
        parsed::ParseProgram,
        ty::{self, TyModule, TyProgram},
    },
    metadata::MetadataManager,
    semantic_analysis::{
        namespace::{self, Package},
        TypeCheckContext,
    },
    BuildConfig, Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_features::ExperimentalFeatures;
use sway_ir::{Context, Module};

use super::{
    symbol_collection_context::SymbolCollectionContext, TypeCheckAnalysis,
    TypeCheckAnalysisContext, TypeCheckFinalization, TypeCheckFinalizationContext,
};

#[derive(Clone, Debug)]
pub struct TypeCheckFailed {
    pub root_module: Option<Arc<TyModule>>,
    pub namespace: Package,
    pub error: ErrorEmitted,
}

impl TyProgram {
    /// Collects the given parsed program to produce a symbol maps.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn collect(
        handler: &Handler,
        engines: &Engines,
        parsed: &ParseProgram,
        namespace: namespace::Namespace,
    ) -> Result<SymbolCollectionContext, ErrorEmitted> {
        let mut ctx = SymbolCollectionContext::new(namespace);
        let ParseProgram { root, kind: _ } = parsed;

        ty::TyModule::collect(handler, engines, &mut ctx, root)?;
        Ok(ctx)
    }

    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
    pub fn type_check(
        handler: &Handler,
        engines: &Engines,
        parsed: &ParseProgram,
        collection_ctx: &mut SymbolCollectionContext,
        mut namespace: namespace::Namespace,
        package_name: &str,
        build_config: Option<&BuildConfig>,
        experimental: ExperimentalFeatures,
    ) -> Result<Self, TypeCheckFailed> {
        let mut ctx =
            TypeCheckContext::from_root(&mut namespace, collection_ctx, engines, experimental)
                .with_kind(parsed.kind);

        let ParseProgram { root, kind } = parsed;

        let root = ty::TyModule::type_check(
            handler,
            ctx.by_ref(),
            engines,
            parsed.kind,
            root,
            build_config,
        )
        .map_err(|error| TypeCheckFailed {
            error,
            root_module: None,
            namespace: ctx.namespace.current_package_ref().clone(),
        })?;

        let experimental = ctx.experimental;
        let (kind, declarations, configurables) =
            Self::validate_root(handler, engines, &root, *kind, package_name, experimental)
                .map_err(|error| TypeCheckFailed {
                    error,
                    root_module: Some(root.clone()),
                    namespace: ctx.namespace.current_package_ref().clone(),
                })?;

        let mut namespace = ctx.namespace().clone();
        Self::validate_coherence(handler, engines, &root, &mut namespace).map_err(|error| {
            TypeCheckFailed {
                error,
                root_module: Some(root.clone()),
                namespace: ctx.namespace.current_package_ref().clone(),
            }
        })?;

        let program = TyProgram {
            kind,
            root_module: (*root).clone(),
            namespace,
            declarations,
            configurables,
            storage_slots: vec![],
            logged_types: vec![],
            messages_types: vec![],
        };

        Ok(program)
    }

    pub(crate) fn get_typed_program_with_initialized_storage_slots(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = engines.de();
        match &self.kind {
            ty::TyProgramKind::Contract { .. } => {
                let storage_decl = self
                    .declarations
                    .iter()
                    .find(|decl| matches!(decl, ty::TyDecl::StorageDecl { .. }));

                // Expecting at most a single storage declaration
                match storage_decl {
                    Some(ty::TyDecl::StorageDecl(ty::StorageDecl { decl_id, .. })) => {
                        let decl = decl_engine.get_storage(decl_id);
                        let mut storage_slots = decl.get_initialized_storage_slots(
                            handler, engines, context, md_mgr, module,
                        )?;
                        // Sort the slots to standardize the output. Not strictly required by the
                        // spec.
                        storage_slots.sort();
                        self.storage_slots = storage_slots;
                        Ok(())
                    }
                    _ => {
                        self.storage_slots = vec![];
                        Ok(())
                    }
                }
            }
            _ => {
                self.storage_slots = vec![];
                Ok(())
            }
        }
    }
}

impl TypeCheckAnalysis for TyProgram {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for node in self.root_module.all_nodes.iter() {
            node.type_check_analyze(handler, ctx)?;
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyProgram {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for node in self.root_module.all_nodes.iter_mut() {
                let _ = node.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}
