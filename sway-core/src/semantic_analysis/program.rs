use crate::{
    error::*,
    language::{parsed::ParseProgram, ty},
    metadata::MetadataManager,
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext,
    },
    Engines,
};
use sway_ir::{Context, Module};

impl ty::TyProgram {
    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        engines: Engines<'_>,
        parsed: &ParseProgram,
        initial_namespace: namespace::Module,
        package_name: &str,
        experimental_storage: bool,
        experimental_private_modules: bool,
    ) -> CompileResult<Self> {
        let mut namespace = Namespace::init_root(initial_namespace);
        let ctx = TypeCheckContext::from_root(&mut namespace, engines)
            .with_kind(parsed.kind.clone())
            .with_experimental_storage(experimental_storage)
            .with_experimental_private_modules(experimental_private_modules);
        let ParseProgram { root, kind } = parsed;
        let mod_res = ty::TyModule::type_check(ctx, root);
        mod_res.flat_map(|root| {
            let res = Self::validate_root(
                engines,
                &root,
                kind.clone(),
                package_name,
                experimental_storage,
            );
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
        engines: Engines<'_>,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
        experimental_storage: bool,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
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
                        let mut storage_slots = check!(
                            decl.get_initialized_storage_slots(
                                engines,
                                context,
                                md_mgr,
                                module,
                                experimental_storage
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors,
                        );
                        // Sort the slots to standardize the output. Not strictly required by the
                        // spec.
                        storage_slots.sort();
                        ok(
                            Self {
                                storage_slots,
                                ..self
                            },
                            warnings,
                            errors,
                        )
                    }
                    _ => ok(
                        Self {
                            storage_slots: vec![],
                            ..self
                        },
                        warnings,
                        errors,
                    ),
                }
            }
            _ => ok(
                Self {
                    storage_slots: vec![],
                    ..self
                },
                warnings,
                errors,
            ),
        }
    }
}
