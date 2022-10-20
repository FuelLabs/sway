use crate::{
    declaration_engine::declaration_engine::de_get_storage,
    error::*,
    language::{parsed::ParseProgram, ty},
    metadata::MetadataManager,
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext,
    },
    type_engine_size,
};
use sway_ir::{Context, Module};
use sway_types::Spanned;

impl ty::TyProgram {
    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        parsed: &ParseProgram,
        initial_namespace: namespace::Module,
    ) -> CompileResult<Self> {
        let mut namespace = Namespace::init_root(initial_namespace);
        let ctx = TypeCheckContext::from_root(&mut namespace);
        let ParseProgram { root, kind } = parsed;
        let mod_span = root.tree.span.clone();
        let mod_res = ty::TyModule::type_check(ctx, root);
        mod_res.flat_map(|root| {
            let kind_res = Self::validate_root(&root, kind.clone(), mod_span);
            kind_res.map(|kind| Self {
                kind,
                root,
                storage_slots: vec![],
                logged_types: vec![],
            })
        })
    }

    pub(crate) fn get_typed_program_with_initialized_storage_slots(
        &self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match &self.kind {
            ty::TyProgramKind::Contract { declarations, .. } => {
                let storage_decl = declarations
                    .iter()
                    .find(|decl| matches!(decl, ty::TyDeclaration::StorageDeclaration(_)));

                // Expecting at most a single storage declaration
                match storage_decl {
                    Some(ty::TyDeclaration::StorageDeclaration(decl_id)) => {
                        let decl = check!(
                            CompileResult::from(de_get_storage(decl_id.clone(), &decl_id.span())),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let mut storage_slots = check!(
                            decl.get_initialized_storage_slots(context, md_mgr, module),
                            return err(warnings, errors),
                            warnings,
                            errors,
                        );
                        // Sort the slots to standardize the output. Not strictly required by the
                        // spec.
                        storage_slots.sort();
                        ok(
                            Self {
                                kind: self.kind.clone(),
                                root: self.root.clone(),
                                storage_slots,
                                logged_types: self.logged_types.clone(),
                            },
                            warnings,
                            errors,
                        )
                    }
                    _ => ok(
                        Self {
                            kind: self.kind.clone(),
                            root: self.root.clone(),
                            storage_slots: vec![],
                            logged_types: self.logged_types.clone(),
                        },
                        warnings,
                        errors,
                    ),
                }
            }
            _ => ok(
                Self {
                    kind: self.kind.clone(),
                    root: self.root.clone(),
                    storage_slots: vec![],
                    logged_types: self.logged_types.clone(),
                },
                warnings,
                errors,
            ),
        }
    }
}
