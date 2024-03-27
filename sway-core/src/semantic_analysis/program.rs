use crate::{
    language::{
        parsed::ParseProgram,
        ty::{self, TyProgram},
    },
    metadata::MetadataManager,
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext,
    },
    BuildConfig, Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{Context, Module};

use super::{
    collection_context::SymbolCollectionContext, TypeCheckAnalysis, TypeCheckAnalysisContext,
    TypeCheckFinalization, TypeCheckFinalizationContext,
};

impl TyProgram {
    /// Collects the given parsed program to produce a symbol map and module evaluation order.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn collect(
        handler: &Handler,
        engines: &Engines,
        parsed: &ParseProgram,
        initial_namespace: namespace::Root,
    ) -> Result<SymbolCollectionContext, ErrorEmitted> {
        let namespace = Namespace::init_root(initial_namespace);
        let mut ctx = SymbolCollectionContext::new(namespace);
        let ParseProgram { root, kind: _ } = parsed;

        ty::TyModule::collect(handler, engines, &mut ctx, root)?;
        Ok(ctx)
    }

    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        handler: &Handler,
        engines: &Engines,
        parsed: &ParseProgram,
        initial_namespace: namespace::Root,
        package_name: &str,
        build_config: Option<&BuildConfig>,
    ) -> Result<Self, ErrorEmitted> {
        let experimental =
            build_config
                .map(|x| x.experimental)
                .unwrap_or(crate::ExperimentalFlags {
                    new_encoding: false,
                });

        let mut namespace = Namespace::init_root(initial_namespace);
        let mut ctx = TypeCheckContext::from_root(&mut namespace, engines, experimental)
            .with_kind(parsed.kind);

        let ParseProgram { root, kind } = parsed;

        let root = ty::TyModule::type_check(handler, ctx.by_ref(), engines, parsed.kind, root)?;

        let (kind, declarations, configurables) = Self::validate_root(
            handler,
            engines,
            &root,
            *kind,
            package_name,
            ctx.experimental,
        )?;

        let program = TyProgram {
            kind,
            root,
            declarations,
            configurables,
            storage_slots: vec![],
            logged_types: vec![],
            messages_types: vec![],
        };

        Ok(program)
    }

    pub(crate) fn get_typed_program_with_initialized_storage_slots(
        self,
        handler: &Handler,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<Self, ErrorEmitted> {
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
                        Ok(Self {
                            storage_slots,
                            ..self
                        })
                    }
                    _ => Ok(Self {
                        storage_slots: vec![],
                        ..self
                    }),
                }
            }
            _ => Ok(Self {
                storage_slots: vec![],
                ..self
            }),
        }
    }
}

impl TypeCheckAnalysis for TyProgram {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for node in self.root.all_nodes.iter() {
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
            for node in self.root.all_nodes.iter_mut() {
                let _ = node.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}
