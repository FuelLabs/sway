use crate::{
    language::{parsed::ParseProgram, ty},
    metadata::MetadataManager,
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext,
    },
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{Context, Module};

impl ty::TyProgram {
    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        handler: &Handler,
        engines: &Engines,
        parsed: &ParseProgram,
        initial_namespace: namespace::Module,
        package_name: &str,
    ) -> Result<Self, ErrorEmitted> {
        let mut namespace = Namespace::init_root(initial_namespace);
        let ctx =
            TypeCheckContext::from_root(&mut namespace, engines).with_kind(parsed.kind.clone());
        let ParseProgram { root, kind } = parsed;
        ty::TyModule::type_check(handler, ctx, root).and_then(|root| {
            let res = Self::validate_root(handler, engines, &root, kind.clone(), package_name);
            res.map(|(kind, declarations, configurables)| Self {
                kind,
                root,
                declarations,
                configurables,
                storage_slots: vec![],
                logged_types: vec![],
                messages_types: vec![],
            })
        })
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
                    Some(ty::TyDecl::StorageDecl(ty::StorageDecl {
                        decl_id,
                        decl_span: _,
                        ..
                    })) => {
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
