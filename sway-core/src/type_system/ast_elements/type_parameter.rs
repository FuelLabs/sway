use crate::{
    decl_engine::{DeclMapping, InterfaceItemMap, ItemMap},
    engine_threading::*,
    has_changes,
    language::{ty, CallPath},
    namespace::TryInsertingTraitImplOnFailure,
    semantic_analysis::{GenericShadowingMode, TypeCheckContext},
    type_system::priv_prelude::*,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct TypeParameter {
    pub type_id: TypeId,
    pub(crate) initial_type_id: TypeId,
    pub name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
    pub(crate) trait_constraints_span: Span,
    pub(crate) is_from_parent: bool,
}

impl HashWithEngines for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TypeParameter {
            type_id,
            name_ident,
            trait_constraints,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = self;
        let type_engine = engines.te();
        type_engine.get(*type_id).hash(state, engines);
        name_ident.hash(state);
        trait_constraints.hash(state, engines);
    }
}

impl EqWithEngines for TypeParameter {}
impl PartialEqWithEngines for TypeParameter {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), ctx)
            && self.name_ident == other.name_ident
            && self.trait_constraints.eq(&other.trait_constraints, ctx)
    }
}

impl OrdWithEngines for TypeParameter {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let TypeParameter {
            type_id: lti,
            name_ident: ln,
            trait_constraints: ltc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = self;
        let TypeParameter {
            type_id: rti,
            name_ident: rn,
            trait_constraints: rtc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = other;
        ln.cmp(rn)
            .then_with(|| {
                ctx.engines()
                    .te()
                    .get(*lti)
                    .cmp(&ctx.engines().te().get(*rti), ctx)
            })
            .then_with(|| ltc.cmp(rtc, ctx))
    }
}

impl SubstTypes for TypeParameter {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.type_id.subst(type_mapping, engines);
            self.trait_constraints.subst(type_mapping, engines);
        }
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl DebugWithEngines for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", self.name_ident)?;
        if !self.trait_constraints.is_empty() {
            write!(
                f,
                ":{}",
                self.trait_constraints
                    .iter()
                    .map(|c| format!("{:?}", engines.help_out(c)))
                    .collect::<Vec<_>>()
                    .join("+")
            )?;
        }
        Ok(())
    }
}

impl fmt::Debug for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(f, "{}: {:?}", self.name_ident, self.type_id);
        for c in &self.trait_constraints {
            let _ = write!(f, "+ {:?}", c.trait_name);
        }
        write!(f, "")
    }
}

impl TypeParameter {
    pub(crate) fn new_self_type(engines: &Engines, span: Span) -> TypeParameter {
        let type_engine = engines.te();

        let name = Ident::new_with_override("Self".into(), span.clone());
        let type_id = type_engine.insert(
            engines,
            TypeInfo::UnknownGeneric {
                name: name.clone(),
                trait_constraints: VecSet(vec![]),
                parent: None,
                is_from_type_parameter: true,
            },
            span.source_id(),
        );
        TypeParameter {
            type_id,
            initial_type_id: type_id,
            name_ident: name,
            trait_constraints: vec![],
            trait_constraints_span: span,
            is_from_parent: false,
        }
    }

    pub(crate) fn insert_self_type_into_namespace(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) {
        let type_parameter_decl =
            ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                name: self.name_ident.clone(),
                type_id: self.type_id,
            });
        let name_a = Ident::new_with_override("self".into(), self.name_ident.span());
        let name_b = Ident::new_with_override("Self".into(), self.name_ident.span());
        let _ = ctx.insert_symbol(handler, name_a, type_parameter_decl.clone());
        let _ = ctx.insert_symbol(handler, name_b, type_parameter_decl);
    }

    /// Type check a list of [TypeParameter] and return a new list of
    /// [TypeParameter]. This will also insert this new list into the current
    /// namespace.
    pub(crate) fn type_check_type_params(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_params: Vec<TypeParameter>,
        self_type_param: Option<TypeParameter>,
    ) -> Result<Vec<TypeParameter>, ErrorEmitted> {
        let mut new_type_params: Vec<TypeParameter> = vec![];

        if let Some(self_type_param) = self_type_param.clone() {
            self_type_param.insert_self_type_into_namespace(handler, ctx.by_ref());
        }

        handler.scope(|handler| {
            for type_param in type_params {
                new_type_params.push(
                    match TypeParameter::type_check(handler, ctx.by_ref(), type_param) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                )
            }

            // Type check trait constraints only after type checking all type parameters.
            // This is required because a trait constraint may use other type parameters.
            // Ex: `struct Struct2<A, B> where A : MyAdd<B>`
            for type_param in &new_type_params {
                TypeParameter::type_check_trait_constraints(handler, ctx.by_ref(), type_param)?;
            }

            Ok(new_type_params)
        })
    }

    // Expands a trait constraint to include all its supertraits.
    // Another way to incorporate this info would be at the level of unification,
    // we would check that two generic type parameters should unify when
    // the left one is a supertrait of the right one (at least in the NonDynamicEquality mode)
    fn expand_trait_constraints(
        handler: &Handler,
        ctx: &TypeCheckContext,
        tc: &TraitConstraint,
    ) -> Vec<TraitConstraint> {
        match ctx
            .namespace()
            .resolve_call_path_typed(handler, ctx.engines, &tc.trait_name, ctx.self_type())
            .ok()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let trait_decl = ctx.engines.de().get_trait(&decl_id);
                let mut result = trait_decl
                    .supertraits
                    .iter()
                    .flat_map(|supertrait| {
                        TypeParameter::expand_trait_constraints(
                            handler,
                            ctx,
                            &TraitConstraint {
                                trait_name: supertrait.name.clone(),
                                type_arguments: tc.type_arguments.clone(),
                            },
                        )
                    })
                    .collect::<Vec<TraitConstraint>>();
                result.push(tc.clone());
                result
            }
            _ => vec![tc.clone()],
        }
    }

    /// Type checks a [TypeParameter] (excluding its [TraitConstraint]s) and
    /// inserts into into the current namespace.
    fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_parameter: TypeParameter,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let TypeParameter {
            initial_type_id,
            name_ident,
            trait_constraints,
            trait_constraints_span,
            is_from_parent,
            type_id,
        } = type_parameter;

        let trait_constraints_with_supertraits: Vec<TraitConstraint> = trait_constraints
            .iter()
            .flat_map(|tc| TypeParameter::expand_trait_constraints(handler, &ctx, tc))
            .collect();

        let parent = if let TypeInfo::UnknownGeneric {
            name: _,
            trait_constraints: _,
            parent,
            is_from_type_parameter: _,
        } = &*type_engine.get(type_id)
        {
            *parent
        } else {
            None
        };

        // Create type id and type parameter before type checking trait constraints.
        // This order is required because a trait constraint may depend on its own type parameter.
        let type_id = type_engine.insert(
            engines,
            TypeInfo::UnknownGeneric {
                name: name_ident.clone(),
                trait_constraints: VecSet(trait_constraints_with_supertraits.clone()),
                parent,
                is_from_type_parameter: true,
            },
            name_ident.span().source_id(),
        );

        let type_parameter = TypeParameter {
            name_ident: name_ident.clone(),
            type_id,
            initial_type_id,
            trait_constraints,
            trait_constraints_span: trait_constraints_span.clone(),
            is_from_parent,
        };

        // Insert the type parameter into the namespace
        type_parameter.insert_into_namespace_self(handler, ctx.by_ref())?;

        Ok(type_parameter)
    }

    /// Type checks a [TypeParameter] [TraitConstraint]s and
    /// inserts them into into the current namespace.
    fn type_check_trait_constraints(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_parameter: &TypeParameter,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let mut trait_constraints_with_supertraits: Vec<TraitConstraint> = type_parameter
            .trait_constraints
            .iter()
            .flat_map(|tc| TypeParameter::expand_trait_constraints(handler, &ctx, tc))
            .collect();

        // Type check the trait constraints.
        for trait_constraint in &mut trait_constraints_with_supertraits {
            trait_constraint.type_check(handler, ctx.by_ref())?;
        }

        // TODO: add check here to see if the type parameter has a valid name and does not have type parameters

        let parent = if let TypeInfo::UnknownGeneric {
            name: _,
            trait_constraints: _,
            parent,
            is_from_type_parameter: _,
        } = &*type_engine.get(type_parameter.type_id)
        {
            *parent
        } else {
            None
        };

        // Trait constraints mutate so we replace the previous type id associated TypeInfo.
        type_engine.replace(
            type_parameter.type_id,
            TypeSourceInfo {
                type_info: TypeInfo::UnknownGeneric {
                    name: type_parameter.name_ident.clone(),
                    trait_constraints: VecSet(trait_constraints_with_supertraits.clone()),
                    parent,
                    is_from_type_parameter: true,
                }
                .into(),
                source_id: type_parameter.name_ident.span().source_id().copied(),
            },
        );

        // Insert the trait constraints into the namespace.
        type_parameter.insert_into_namespace_constraints(handler, ctx.by_ref())?;

        Ok(())
    }

    pub fn insert_into_namespace(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        self.insert_into_namespace_constraints(handler, ctx.by_ref())?;

        self.insert_into_namespace_self(handler, ctx.by_ref())?;

        Ok(())
    }

    fn insert_into_namespace_constraints(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        // Insert the trait constraints into the namespace.
        for trait_constraint in &self.trait_constraints {
            TraitConstraint::insert_into_namespace(
                handler,
                ctx.by_ref(),
                self.type_id,
                trait_constraint,
            )?;
        }

        Ok(())
    }

    pub(crate) fn insert_into_namespace_self(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        let Self {
            is_from_parent,
            name_ident,
            type_id,
            ..
        } = self;

        if *is_from_parent {
            ctx = ctx.with_generic_shadowing_mode(GenericShadowingMode::Allow);

            let sy = ctx
                .namespace()
                .module(ctx.engines())
                .current_items()
                .symbols
                .get(name_ident)
                .unwrap();

            match sy.expect_typed_ref() {
                ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                    type_id: parent_type_id,
                    ..
                }) => {
                    if let TypeInfo::UnknownGeneric {
                        name,
                        trait_constraints,
                        parent,
                        is_from_type_parameter,
                    } = &*ctx.engines().te().get(*type_id)
                    {
                        if parent.is_some() {
                            return Ok(());
                        }

                        ctx.engines.te().replace(
                            *type_id,
                            TypeSourceInfo {
                                type_info: TypeInfo::UnknownGeneric {
                                    name: name.clone(),
                                    trait_constraints: trait_constraints.clone(),
                                    parent: Some(*parent_type_id),
                                    is_from_type_parameter: *is_from_type_parameter,
                                }
                                .into(),
                                source_id: name.span().source_id().copied(),
                            },
                        );
                    }
                }
                _ => {
                    handler.emit_err(CompileError::Internal(
                        "Unexpected TyDeclaration for TypeParameter.",
                        self.name_ident.span(),
                    ));
                }
            }
        }

        // Insert the type parameter into the namespace as a dummy type
        // declaration.
        let type_parameter_decl =
            ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                name: name_ident.clone(),
                type_id: *type_id,
            });
        ctx.insert_symbol(handler, name_ident.clone(), type_parameter_decl)
            .ok();

        Ok(())
    }

    /// Creates a [DeclMapping] from a list of [TypeParameter]s.
    /// `function_name` and `access_span` are used only for error reporting.
    pub(crate) fn gather_decl_mapping_from_trait_constraints(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_parameters: &[TypeParameter],
        function_name: &str,
        access_span: &Span,
    ) -> Result<DeclMapping, ErrorEmitted> {
        let mut interface_item_refs: InterfaceItemMap = BTreeMap::new();
        let mut item_refs: ItemMap = BTreeMap::new();
        let mut impld_item_refs: ItemMap = BTreeMap::new();
        let engines = ctx.engines();

        handler.scope(|handler| {
            for type_param in type_parameters {
                let TypeParameter {
                    type_id,
                    trait_constraints,
                    ..
                } = type_param;

                // Check to see if the trait constraints are satisfied.
                match ctx
                    .namespace_mut()
                    .module_mut(engines)
                    .current_items_mut()
                    .implemented_traits
                    .check_if_trait_constraints_are_satisfied_for_type(
                        handler,
                        *type_id,
                        trait_constraints,
                        access_span,
                        engines,
                        TryInsertingTraitImplOnFailure::Yes,
                    ) {
                    Ok(res) => res,
                    Err(_) => continue,
                }

                for trait_constraint in trait_constraints {
                    let TraitConstraint {
                        trait_name,
                        type_arguments: trait_type_arguments,
                    } = trait_constraint;

                    let (trait_interface_item_refs, trait_item_refs, trait_impld_item_refs) =
                        match handle_trait(
                            handler,
                            &ctx,
                            *type_id,
                            trait_name,
                            trait_type_arguments,
                            function_name,
                            access_span.clone(),
                        ) {
                            Ok(res) => res,
                            Err(_) => continue,
                        };
                    interface_item_refs.extend(trait_interface_item_refs);
                    item_refs.extend(trait_item_refs);
                    impld_item_refs.extend(trait_impld_item_refs);
                }
            }

            let decl_mapping = DeclMapping::from_interface_and_item_and_impld_decl_refs(
                interface_item_refs,
                item_refs,
                impld_item_refs,
            );
            Ok(decl_mapping)
        })
    }
}

fn handle_trait(
    handler: &Handler,
    ctx: &TypeCheckContext,
    type_id: TypeId,
    trait_name: &CallPath,
    type_arguments: &[TypeArgument],
    function_name: &str,
    access_span: Span,
) -> Result<(InterfaceItemMap, ItemMap, ItemMap), ErrorEmitted> {
    let engines = ctx.engines;
    let decl_engine = engines.de();

    let mut interface_item_refs: InterfaceItemMap = BTreeMap::new();
    let mut item_refs: ItemMap = BTreeMap::new();
    let mut impld_item_refs: ItemMap = BTreeMap::new();

    handler.scope(|handler| {
        match ctx
            .namespace()
            // Use the default Handler to avoid emitting the redundant SymbolNotFound error.
            .resolve_call_path_typed(&Handler::default(), engines, trait_name, ctx.self_type())
            .ok()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let trait_decl = decl_engine.get_trait(&decl_id);

                let (trait_interface_item_refs, trait_item_refs, trait_impld_item_refs) =
                    trait_decl.retrieve_interface_surface_and_items_and_implemented_items_for_type(
                        ctx,
                        type_id,
                        trait_name,
                        type_arguments,
                    );
                interface_item_refs.extend(trait_interface_item_refs);
                item_refs.extend(trait_item_refs);
                impld_item_refs.extend(trait_impld_item_refs);

                for supertrait in &trait_decl.supertraits {
                    let (
                        supertrait_interface_item_refs,
                        supertrait_item_refs,
                        supertrait_impld_item_refs,
                    ) = match handle_trait(
                        handler,
                        ctx,
                        type_id,
                        &supertrait.name,
                        &[],
                        function_name,
                        access_span.clone(),
                    ) {
                        Ok(res) => res,
                        Err(_) => continue,
                    };
                    interface_item_refs.extend(supertrait_interface_item_refs);
                    item_refs.extend(supertrait_item_refs);
                    impld_item_refs.extend(supertrait_impld_item_refs);
                }
            }
            _ => {
                let trait_candidates = decl_engine
                    .get_traits_by_name(&trait_name.suffix)
                    .iter()
                    .map(|trait_decl| {
                        // In the case of an internal library, always add :: to the candidate call path.
                        let import_path = trait_decl
                            .call_path
                            .to_import_path(ctx.engines(), ctx.namespace());
                        if import_path == trait_decl.call_path {
                            // If external library.
                            import_path.to_string()
                        } else {
                            format!("::{import_path}")
                        }
                    })
                    .collect();

                handler.emit_err(CompileError::TraitNotImportedAtFunctionApplication {
                    trait_name: trait_name.suffix.to_string(),
                    function_name: function_name.to_string(),
                    function_call_site_span: access_span.clone(),
                    trait_constraint_span: trait_name.suffix.span(),
                    trait_candidates,
                });
            }
        }

        Ok((interface_item_refs, item_refs, impld_item_refs))
    })
}
