use crate::{
    abi_generation::abi_str::AbiStrContext,
    decl_engine::{
        parsed_id::ParsedDeclId, DeclEngineGet, DeclEngineInsert as _, DeclMapping,
        InterfaceItemMap, ItemMap, MaterializeConstGenerics, ParsedDeclEngineGet as _,
    },
    engine_threading::*,
    has_changes,
    language::{
        parsed::ConstGenericDeclaration,
        ty::{
            self, ConstGenericDecl, ConstantDecl, TyConstGenericDecl, TyConstantDecl, TyExpression,
            TyExpressionVariant,
        },
        CallPath, CallPathType,
    },
    namespace::TraitMap,
    semantic_analysis::{GenericShadowingMode, TypeCheckContext},
    type_system::priv_prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{ident::Ident, span::Span, BaseIdent, Named, Spanned};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeParameter {
    Type(GenericTypeParameter),
    Const(ConstGenericParameter),
}

impl TypeParameter {
    pub(crate) fn insert_into_namespace_constraints(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        // Insert the trait constraints into the namespace.
        match self {
            TypeParameter::Type(p) => {
                for trait_constraint in &p.trait_constraints {
                    TraitConstraint::insert_into_namespace(
                        handler,
                        ctx.by_ref(),
                        p.type_id,
                        trait_constraint,
                    )?;
                }
            }
            TypeParameter::Const(_) => {}
        }

        Ok(())
    }

    pub(crate) fn insert_into_namespace_self(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        let (is_from_parent, name, type_id, ty_decl) = match self {
            TypeParameter::Type(GenericTypeParameter {
                is_from_parent,
                name,
                type_id,
                ..
            }) => (
                is_from_parent,
                name,
                *type_id,
                ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                    name: name.clone(),
                    type_id: *type_id,
                }),
            ),
            TypeParameter::Const(ConstGenericParameter {
                is_from_parent,
                name,
                id,
                span,
                ty,
                ..
            }) => {
                let decl_ref = ctx.engines.de().insert(
                    TyConstGenericDecl {
                        call_path: CallPath {
                            prefixes: vec![],
                            suffix: name.clone(),
                            callpath_type: CallPathType::Ambiguous,
                        },
                        span: span.clone(),
                        return_type: *ty,
                        value: None,
                    },
                    id.as_ref(),
                );
                (
                    is_from_parent,
                    name,
                    ctx.engines.te().id_of_u64(),
                    ty::TyDecl::ConstGenericDecl(ConstGenericDecl {
                        decl_id: *decl_ref.id(),
                    }),
                )
            }
        };

        if *is_from_parent {
            ctx = ctx.with_generic_shadowing_mode(GenericShadowingMode::Allow);

            let (resolve_declaration, _) =
                ctx.namespace()
                    .current_module()
                    .resolve_symbol(handler, ctx.engines(), name)?;

            match resolve_declaration.expect_typed_ref() {
                ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                    type_id: parent_type_id,
                    ..
                }) => {
                    if let TypeInfo::UnknownGeneric {
                        name,
                        trait_constraints,
                        parent,
                        is_from_type_parameter,
                    } = &*ctx.engines().te().get(type_id)
                    {
                        if parent.is_some() {
                            return Ok(());
                        }

                        ctx.engines.te().replace(
                            ctx.engines(),
                            type_id,
                            TypeInfo::UnknownGeneric {
                                name: name.clone(),
                                trait_constraints: trait_constraints.clone(),
                                parent: Some(*parent_type_id),
                                is_from_type_parameter: *is_from_type_parameter,
                            },
                        );
                    }
                }
                ty::TyDecl::ConstGenericDecl(_) => {}
                _ => {
                    handler.emit_err(CompileError::Internal(
                        "Unexpected TyDeclaration for TypeParameter.",
                        name.span(),
                    ));
                }
            }
        }

        // Insert the type parameter into the namespace as a dummy type
        // declaration.
        ctx.insert_symbol(handler, name.clone(), ty_decl).ok();

        Ok(())
    }

    pub(crate) fn unifies(
        &self,
        type_id: TypeId,
        decider: impl Fn(TypeId, TypeId) -> bool,
    ) -> bool {
        match self {
            TypeParameter::Type(generic_type_parameter) => {
                decider(type_id, generic_type_parameter.type_id)
            }
            TypeParameter::Const(const_generic_parameter) => {
                decider(type_id, const_generic_parameter.ty)
            }
        }
    }
}

impl Named for TypeParameter {
    fn name(&self) -> &BaseIdent {
        match self {
            TypeParameter::Type(p) => &p.name,
            TypeParameter::Const(p) => &p.name,
        }
    }
}

impl EqWithEngines for TypeParameter {}
impl PartialEqWithEngines for TypeParameter {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (TypeParameter::Type(l), TypeParameter::Type(r)) => l.eq(r, ctx),
            (TypeParameter::Const(l), TypeParameter::Const(r)) => {
                <ConstGenericParameter as PartialEqWithEngines>::eq(l, r, ctx)
            }
            _ => false,
        }
    }
}

impl HashWithEngines for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        std::mem::discriminant(self).hash(state);
        match self {
            TypeParameter::Type(p) => p.hash(state, engines),
            TypeParameter::Const(p) => p.hash(state, engines),
        }
    }
}

impl SubstTypes for TypeParameter {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        match self {
            TypeParameter::Type(p) => p.subst_inner(ctx),
            TypeParameter::Const(p) => p.subst_inner(ctx),
        }
    }
}

impl IsConcrete for TypeParameter {
    fn is_concrete(&self, engines: &Engines) -> bool {
        match self {
            TypeParameter::Type(p) => p.is_concrete(engines),
            TypeParameter::Const(p) => p.is_concrete(engines),
        }
    }
}

impl OrdWithEngines for TypeParameter {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        match (self, other) {
            (TypeParameter::Type(l), TypeParameter::Type(r)) => l.cmp(r, ctx),
            (TypeParameter::Const(l), TypeParameter::Const(r)) => l.cmp(r),
            _ => todo!(),
        }
    }
}

impl DebugWithEngines for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let s = match self {
            TypeParameter::Type(p) => {
                format!(
                    "{:?} -> {:?}",
                    engines.help_out(p.initial_type_id),
                    engines.help_out(p.type_id)
                )
            }
            TypeParameter::Const(p) => match p.expr.as_ref() {
                Some(ConstGenericExpr::Literal { val, .. }) => format!("{} -> {}", p.name, val),
                Some(ConstGenericExpr::AmbiguousVariableExpression { ident, .. }) => {
                    format!("{ident}")
                }
                None => format!("{} -> None", p.name),
            },
        };
        write!(f, "{s}")
    }
}

impl TypeParameter {
    pub fn as_type_parameter(&self) -> Option<&GenericTypeParameter> {
        match self {
            TypeParameter::Type(p) => Some(p),
            TypeParameter::Const(_) => None,
        }
    }

    pub fn as_type_parameter_mut(&mut self) -> Option<&mut GenericTypeParameter> {
        match self {
            TypeParameter::Type(p) => Some(p),
            TypeParameter::Const(_) => None,
        }
    }

    pub fn abi_str(
        &self,
        handler: &Handler,
        engines: &Engines,
        ctx: &AbiStrContext,
        is_root: bool,
    ) -> Result<String, ErrorEmitted> {
        match self {
            TypeParameter::Type(p) => engines
                .te()
                .get(p.type_id)
                .abi_str(handler, ctx, engines, is_root),
            TypeParameter::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }

    pub fn as_const_parameter(&self) -> Option<&ConstGenericParameter> {
        match self {
            TypeParameter::Type(_) => None,
            TypeParameter::Const(p) => Some(p),
        }
    }

    pub fn as_const_parameter_mut(&mut self) -> Option<&mut ConstGenericParameter> {
        match self {
            TypeParameter::Type(_) => None,
            TypeParameter::Const(p) => Some(p),
        }
    }

    pub fn is_from_parent(&self) -> bool {
        match self {
            TypeParameter::Type(p) => p.is_from_parent,
            TypeParameter::Const(p) => p.is_from_parent,
        }
    }

    pub fn with_is_from_parent(self, is_from_parent: bool) -> Self {
        match self {
            TypeParameter::Type(mut p) => {
                p.is_from_parent = is_from_parent;
                TypeParameter::Type(p)
            }
            TypeParameter::Const(mut p) => {
                p.is_from_parent = is_from_parent;
                TypeParameter::Const(p)
            }
        }
    }
}

/// [GenericTypeParameter] describes a generic type parameter, including its
/// monomorphized version. It holds the `name` of the parameter, its
/// `type_id`, and the `initial_type_id`, as well as an additional
/// information about that type parameter, called the annotation.
///
/// If a [GenericTypeParameter] is considered as not being annotated,
/// its `initial_type_id` must be same as `type_id`, its
/// `trait_constraints_span` must be [Span::dummy]
/// and its `is_from_parent` must be false.
///
/// The annotations are ignored when calculating the [GenericTypeParameter]'s hash
/// (with engines) and equality (with engines).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeParameter {
    pub type_id: TypeId,
    /// Denotes the initial type represented by the [GenericTypeParameter], before
    /// unification, monomorphization, or replacement of [TypeInfo::Custom]s.
    pub(crate) initial_type_id: TypeId,
    pub name: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
    pub(crate) trait_constraints_span: Span,
    pub(crate) is_from_parent: bool,
}

impl GenericTypeParameter {
    /// Returns true if `self` is annotated by heaving either
    /// its [Self::initial_type_id] different from [Self::type_id],
    /// or [Self::trait_constraints_span] different from [Span::dummy]
    /// or [Self::is_from_parent] different from false.
    pub fn is_annotated(&self) -> bool {
        self.type_id != self.initial_type_id
            || self.is_from_parent
            || !self.trait_constraints_span.is_dummy()
    }
}

impl HashWithEngines for GenericTypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let GenericTypeParameter {
            type_id,
            name,
            trait_constraints,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = self;
        let type_engine = engines.te();
        type_engine.get(*type_id).hash(state, engines);
        name.hash(state);
        trait_constraints.hash(state, engines);
    }
}

impl EqWithEngines for GenericTypeParameter {}
impl PartialEqWithEngines for GenericTypeParameter {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), ctx)
            && self.name == other.name
            && self.trait_constraints.eq(&other.trait_constraints, ctx)
    }
}

impl OrdWithEngines for GenericTypeParameter {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let GenericTypeParameter {
            type_id: lti,
            name: lname,
            trait_constraints: ltc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = &self;
        let GenericTypeParameter {
            type_id: rti,
            name: rn,
            trait_constraints: rtc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = &other;
        let type_engine = ctx.engines().te();
        let ltype = type_engine.get(*lti);
        let rtype = type_engine.get(*rti);
        ltype
            .cmp(&rtype, ctx)
            .then_with(|| lname.cmp(rn).then_with(|| ltc.cmp(rtc, ctx)))
    }
}

impl SubstTypes for GenericTypeParameter {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.type_id.subst(ctx);
            self.trait_constraints.subst(ctx);
        }
    }
}

impl Spanned for GenericTypeParameter {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl IsConcrete for GenericTypeParameter {
    fn is_concrete(&self, engines: &Engines) -> bool {
        self.type_id.is_concrete(engines, TreatNumericAs::Concrete)
    }
}

impl DebugWithEngines for GenericTypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", self.name)?;
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

impl GenericTypeParameter {
    /// Creates a new [GenericTypeParameter] that represents a `Self` type.
    /// The returned type parameter will have its [GenericTypeParameter::name]
    /// set to "Self" with the provided `use_site_span`.
    ///
    /// `Self` type is a [TypeInfo::UnknownGeneric] and therefore [GenericTypeParameter::type_id]s
    /// will be set to newly created unknown generic type.
    ///
    /// Note that the span in general does not point to a reserved word "Self" in
    /// the source code, nor is related to it. The `Self` type represents the type
    /// in `impl`s and does not necessarily relate to the "Self" keyword in code.
    ///
    /// Therefore, *the span must always point to a location in the source file in which
    /// the particular `Self` type is, e.g., being declared or implemented*.
    pub(crate) fn new_self_type(engines: &Engines, use_site_span: Span) -> GenericTypeParameter {
        let type_engine = engines.te();

        let (type_id, name) = type_engine.new_unknown_generic_self(use_site_span, true);
        GenericTypeParameter {
            type_id,
            initial_type_id: type_id,
            name,
            trait_constraints: vec![],
            trait_constraints_span: Span::dummy(),
            is_from_parent: false,
        }
    }

    /// Creates a new [TypeParameter] specifically to be used as the type parameter
    /// for a [TypeInfo::Placeholder]. The returned type parameter will have its
    /// [TypeParameter::name] set to "_" with the provided `placeholder_or_use_site_span`
    /// and its [TypeParameter::type_id]s set to the `type_id`.
    ///
    /// Note that in the user written code, the span will always point to the place in
    /// the source code where "_" is located. In the compiler generated code that is not always the case
    /// be the case. For cases when the span does not point to "_" see the comments
    /// in the usages of this method.
    ///
    /// However, *the span must always point to a location in the source file in which
    /// the particular placeholder is considered to be used*.
    pub(crate) fn new_placeholder(
        type_id: TypeId,
        placeholder_or_use_site_span: Span,
    ) -> GenericTypeParameter {
        GenericTypeParameter {
            type_id,
            initial_type_id: type_id,
            name: BaseIdent::new_with_override("_".into(), placeholder_or_use_site_span),
            trait_constraints: vec![],
            trait_constraints_span: Span::dummy(),
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
                name: self.name.clone(),
                type_id: self.type_id,
            });
        let name_a = Ident::new_with_override("self".into(), self.name.span());
        let name_b = Ident::new_with_override("Self".into(), self.name.span());
        let _ = ctx.insert_symbol(handler, name_a, type_parameter_decl.clone());
        let _ = ctx.insert_symbol(handler, name_b, type_parameter_decl);
    }

    /// Type check a list of [TypeParameter] and return a new list of
    /// [TypeParameter]. This will also insert this new list into the current
    /// namespace.
    pub(crate) fn type_check_type_params(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        generic_params: Vec<TypeParameter>,
        self_type_param: Option<GenericTypeParameter>,
    ) -> Result<Vec<TypeParameter>, ErrorEmitted> {
        let mut new_generic_params: Vec<TypeParameter> = vec![];

        if let Some(self_type_param) = self_type_param.clone() {
            self_type_param.insert_self_type_into_namespace(handler, ctx.by_ref());
        }

        handler.scope(|handler| {
            let mut already_declared = HashMap::new();
            for p in generic_params {
                let p = match p {
                    TypeParameter::Type(p) => {
                        match GenericTypeParameter::type_check(handler, ctx.by_ref(), p) {
                            Ok(res) => res,
                            Err(_) => continue,
                        }
                    }
                    TypeParameter::Const(p) => {
                        if let Some(old) = already_declared.insert(p.name.clone(), p.span.clone()) {
                            let (old, new) = if old < p.span {
                                (old, p.span.clone())
                            } else {
                                (p.span.clone(), old)
                            };
                            handler.emit_err(CompileError::MultipleDefinitionsOfConstant {
                                name: p.name.clone(),
                                new,
                                old,
                            });
                        }
                        TypeParameter::Const(p.clone())
                    }
                };
                p.insert_into_namespace_self(handler, ctx.by_ref())?;
                new_generic_params.push(p)
            }

            // Type check trait constraints only after type checking all type parameters.
            // This is required because a trait constraint may use other type parameters.
            // Ex: `struct Struct2<A, B> where A : MyAdd<B>`
            for type_parameter in new_generic_params.iter_mut() {
                match type_parameter {
                    TypeParameter::Type(type_parameter) => {
                        GenericTypeParameter::type_check_trait_constraints(
                            handler,
                            ctx.by_ref(),
                            type_parameter,
                        )?;
                    }
                    TypeParameter::Const(_) => {}
                }

                type_parameter.insert_into_namespace_constraints(handler, ctx.by_ref())?;
            }

            Ok(new_generic_params)
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
        match ctx.resolve_call_path(handler, &tc.trait_name).ok() {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let trait_decl = ctx.engines.de().get_trait(&decl_id);
                let mut result = vec![tc.clone()];
                result.extend(
                    trait_decl
                        .supertraits
                        .iter()
                        .flat_map(|supertrait| {
                            GenericTypeParameter::expand_trait_constraints(
                                handler,
                                ctx,
                                &TraitConstraint {
                                    trait_name: supertrait.name.clone(),
                                    type_arguments: tc.type_arguments.clone(),
                                },
                            )
                        })
                        .collect::<Vec<TraitConstraint>>(),
                );
                result
            }
            _ => vec![tc.clone()],
        }
    }

    /// Type checks a [TypeParameter] (excluding its [TraitConstraint]s) and
    /// inserts into into the current namespace.
    fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        type_parameter: GenericTypeParameter,
    ) -> Result<TypeParameter, ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let GenericTypeParameter {
            initial_type_id,
            name,
            trait_constraints,
            trait_constraints_span,
            is_from_parent,
            type_id,
        } = type_parameter;

        let trait_constraints_with_supertraits: Vec<TraitConstraint> = trait_constraints
            .iter()
            .flat_map(|tc| GenericTypeParameter::expand_trait_constraints(handler, &ctx, tc))
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
        let type_id = type_engine.new_unknown_generic(
            name.clone(),
            VecSet(trait_constraints_with_supertraits.clone()),
            parent,
            true,
        );

        let type_parameter = GenericTypeParameter {
            name,
            type_id,
            initial_type_id,
            trait_constraints,
            trait_constraints_span: trait_constraints_span.clone(),
            is_from_parent,
        };

        // Insert the type parameter into the namespace
        Ok(TypeParameter::Type(type_parameter))
    }

    /// Type checks a [TypeParameter] [TraitConstraint]s and
    /// inserts them into into the current namespace.
    fn type_check_trait_constraints(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_parameter: &mut GenericTypeParameter,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let mut trait_constraints_with_supertraits: Vec<TraitConstraint> = type_parameter
            .trait_constraints
            .iter()
            .flat_map(|tc| GenericTypeParameter::expand_trait_constraints(handler, &ctx, tc))
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
            ctx.engines(),
            type_parameter.type_id,
            TypeInfo::UnknownGeneric {
                name: type_parameter.name.clone(),
                trait_constraints: VecSet(trait_constraints_with_supertraits.clone()),
                parent,
                is_from_type_parameter: true,
            },
        );

        type_parameter.trait_constraints = trait_constraints_with_supertraits;

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
            for type_param in type_parameters.iter().filter_map(|x| x.as_type_parameter()) {
                let GenericTypeParameter {
                    type_id,
                    trait_constraints,
                    ..
                } = type_param;

                let code_block_first_pass = ctx.code_block_first_pass();
                if !code_block_first_pass {
                    // Tries to unify type id with a single existing trait implementation.
                    // If more than one implementation exists we throw an error.
                    // We only try to do the type inference from trait with a single trait constraint.
                    if !type_id.is_concrete(engines, TreatNumericAs::Concrete) && trait_constraints.len() == 1 {
                        let concrete_trait_type_ids : Vec<(TypeId, String)>=
                            TraitMap::get_trait_constraints_are_satisfied_for_types(
                                ctx
                            .namespace()
                            .current_module(), handler, *type_id, trait_constraints, engines,
                            )?
                            .into_iter()
                            .filter_map(|t| {
                                if t.0.is_concrete(engines, TreatNumericAs::Concrete) {
                                    Some(t)
                                } else {
                                    None
                                }
                            }).collect();

                        match concrete_trait_type_ids.len().cmp(&1) {
                            Ordering::Equal => {
                                ctx.engines.te().unify_with_generic(
                                    handler,
                                    engines,
                                    *type_id,
                                    concrete_trait_type_ids.first().unwrap().0,
                                    access_span,
                                    "Type parameter type does not match up with matched trait implementing type.",
                                    || None,
                                );
                            }
                            Ordering::Greater => {
                                return Err(handler.emit_err(CompileError::MultipleImplsSatisfyingTraitForType{
                                    span:access_span.clone(),
                                    type_annotation: engines.help_out(type_id).to_string(),
                                    trait_names: trait_constraints.iter().map(|t| t.to_display_name(engines, ctx.namespace())).collect(),
                                    trait_types_and_names: concrete_trait_type_ids.iter().map(|t| (engines.help_out(t.0).to_string(), t.1.clone())).collect::<Vec<_>>()
                                }));
                            }
                            Ordering::Less => {
                            }
                        }
                    }
                    // Check to see if the trait constraints are satisfied.
                    match TraitMap::check_if_trait_constraints_are_satisfied_for_type(
                            handler,
                            ctx.namespace_mut().current_module_mut(),
                            *type_id,
                            trait_constraints,
                            access_span,
                            engines,
                        ) {
                        Ok(res) => {
                            res
                        },
                        Err(_) => {
                            continue
                        },
                    }
                }

                for trait_constraint in trait_constraints {
                    let TraitConstraint {
                        trait_name,
                        type_arguments: trait_type_arguments,
                    } = trait_constraint;

                    let Ok((mut trait_interface_item_refs, mut trait_item_refs, mut trait_impld_item_refs)) = handle_trait(
                        handler,
                        &ctx,
                        *type_id,
                        trait_name,
                        trait_type_arguments,
                        function_name,
                        access_span.clone(),
                    ) else {
                        continue;
                    };

                    interface_item_refs.append(&mut trait_interface_item_refs);
                    item_refs.append(&mut trait_item_refs);
                    impld_item_refs.append(&mut trait_impld_item_refs);
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
    type_arguments: &[GenericArgument],
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
            // Use the default Handler to avoid emitting the redundant SymbolNotFound error.
            .resolve_call_path(&Handler::default(), trait_name)
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
                        // TODO: Replace with a call to a dedicated `CallPath` method
                        //       once https://github.com/FuelLabs/sway/issues/6873 is fixed.
                        let full_path = trait_decl
                            .call_path
                            .to_fullpath(ctx.engines(), ctx.namespace());
                        if ctx.namespace().module_is_external(&full_path.prefixes) {
                            full_path.to_string()
                        } else {
                            let import_path = trait_decl
                                .call_path
                                .to_import_path(ctx.engines(), ctx.namespace());
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstGenericExprTyDecl {
    ConstGenericDecl(ConstGenericDecl),
    ConstantDecl(ConstantDecl),
}

impl MaterializeConstGenerics for ConstGenericExprTyDecl {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &sway_error::handler::Handler,
        name: &str,
        value: &crate::language::ty::TyExpression,
    ) -> Result<(), sway_error::handler::ErrorEmitted> {
        match self {
            ConstGenericExprTyDecl::ConstGenericDecl(decl) => {
                let mut decl = TyConstGenericDecl::clone(&*engines.de().get(&decl.decl_id));
                decl.materialize_const_generics(engines, handler, name, value)?;

                let decl_ref = engines.de().insert(decl, None); // TODO improve parsed_decl_id
                *self = ConstGenericExprTyDecl::ConstGenericDecl(ConstGenericDecl {
                    decl_id: *decl_ref.id(),
                });
                Ok(())
            }
            ConstGenericExprTyDecl::ConstantDecl(decl) => {
                let mut decl = TyConstantDecl::clone(&*engines.de().get(&decl.decl_id));
                decl.materialize_const_generics(engines, handler, name, value)?;

                let decl_ref = engines.de().insert(decl, None); // TODO improve parsed_decl_id
                *self = ConstGenericExprTyDecl::ConstantDecl(ConstantDecl {
                    decl_id: *decl_ref.id(),
                });
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstGenericExpr {
    Literal {
        val: usize,
        span: Span,
    },
    AmbiguousVariableExpression {
        ident: Ident,
        decl: Option<ConstGenericExprTyDecl>,
    },
}

impl ConstGenericExpr {
    pub fn from_ty_expression(
        handler: &Handler,
        expr: &ty::TyExpression,
    ) -> Result<Self, ErrorEmitted> {
        match &expr.expression {
            ty::TyExpressionVariant::Literal(crate::language::Literal::U64(val)) => {
                Ok(ConstGenericExpr::Literal {
                    val: *val as usize,
                    span: expr.span.clone(),
                })
            }
            ty::TyExpressionVariant::ConstGenericExpression { call_path, .. } => {
                Ok(ConstGenericExpr::AmbiguousVariableExpression {
                    ident: call_path.suffix.clone(),
                    decl: None,
                })
            }
            ty::TyExpressionVariant::ConstantExpression { decl, .. } => {
                Ok(ConstGenericExpr::AmbiguousVariableExpression {
                    ident: decl.call_path.suffix.clone(),
                    decl: None,
                })
            }
            _ => Err(
                handler.emit_err(CompileError::ConstGenericNotSupportedHere {
                    span: expr.span.clone(),
                }),
            ),
        }
    }

    pub fn to_ty_expression(&self, engines: &Engines) -> TyExpression {
        match self {
            ConstGenericExpr::Literal { val, span } => TyExpression {
                expression: ty::TyExpressionVariant::Literal(crate::language::Literal::U64(
                    *val as u64,
                )),
                return_type: engines.te().id_of_u64(),
                span: span.clone(),
            },
            ConstGenericExpr::AmbiguousVariableExpression { ident, decl } => {
                let expression = match decl {
                    Some(ConstGenericExprTyDecl::ConstGenericDecl(decl)) => {
                        TyExpressionVariant::ConstGenericExpression {
                            decl: Box::new(TyConstGenericDecl::clone(
                                &*engines.de().get(&decl.decl_id),
                            )),
                            span: ident.span(),
                            call_path: CallPath {
                                prefixes: vec![],
                                suffix: ident.clone(),
                                callpath_type: CallPathType::Ambiguous,
                            },
                        }
                    }
                    Some(ConstGenericExprTyDecl::ConstantDecl(decl)) => {
                        TyExpressionVariant::ConstantExpression {
                            decl: Box::new(TyConstantDecl::clone(
                                &*engines.de().get(&decl.decl_id),
                            )),
                            span: ident.span(),
                            call_path: Some(CallPath {
                                prefixes: vec![],
                                suffix: ident.clone(),
                                callpath_type: CallPathType::Ambiguous,
                            }),
                        }
                    }
                    None => {
                        unreachable!("Type check guarantee this variable points to a know decl")
                    }
                };
                TyExpression {
                    expression,
                    return_type: engines.te().id_of_u64(),
                    span: ident.span().clone(),
                }
            }
        }
    }

    pub fn discriminant_value(&self) -> usize {
        match &self {
            Self::Literal { .. } => 0,
            Self::AmbiguousVariableExpression { .. } => 1,
        }
    }

    /// Creates a new literal [Length] without span annotation.
    pub fn literal(val: usize, span: Option<Span>) -> Self {
        Self::Literal {
            val,
            span: span.unwrap_or(Span::dummy()),
        }
    }

    pub fn as_literal_val(&self) -> Option<usize> {
        match self {
            Self::Literal { val, .. } => Some(*val),
            _ => None,
        }
    }

    pub fn is_annotated(&self) -> bool {
        !self.span().is_dummy()
    }
}

impl PartialOrd for ConstGenericExpr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Literal { val: l, .. }, Self::Literal { val: r, .. }) => l.partial_cmp(r),
            (
                Self::AmbiguousVariableExpression { ident: l, .. },
                Self::AmbiguousVariableExpression { ident: r, .. },
            ) => l.partial_cmp(r),
            _ => None,
        }
    }
}

impl Ord for ConstGenericExpr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Literal { val: l, .. }, Self::Literal { val: r, .. }) => l.cmp(r),
            (
                Self::AmbiguousVariableExpression { ident: l, .. },
                Self::AmbiguousVariableExpression { ident: r, .. },
            ) => l.cmp(r),
            (
                ConstGenericExpr::Literal { .. },
                ConstGenericExpr::AmbiguousVariableExpression { .. },
            ) => Ordering::Less,
            (
                ConstGenericExpr::AmbiguousVariableExpression { .. },
                ConstGenericExpr::Literal { .. },
            ) => Ordering::Greater,
        }
    }
}

impl Eq for ConstGenericExpr {}

impl PartialEq for ConstGenericExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal { val: l, .. }, Self::Literal { val: r, .. }) => l == r,
            (
                Self::AmbiguousVariableExpression { ident: l, .. },
                Self::AmbiguousVariableExpression { ident: r, .. },
            ) => l == r,
            _ => false,
        }
    }
}

impl std::hash::Hash for ConstGenericExpr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Self::Literal { val, .. } => val.hash(state),
            Self::AmbiguousVariableExpression { ident, .. } => ident.hash(state),
        }
    }
}

impl Spanned for ConstGenericExpr {
    fn span(&self) -> Span {
        match self {
            Self::Literal { span, .. } => span.clone(),
            Self::AmbiguousVariableExpression { ident, .. } => ident.span(),
        }
    }
}

impl DebugWithEngines for ConstGenericExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &crate::Engines) -> std::fmt::Result {
        match self {
            Self::Literal { val, .. } => write!(f, "{val}"),
            Self::AmbiguousVariableExpression { ident, .. } => {
                write!(f, "{}", ident.as_str())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstGenericParameter {
    pub name: Ident,
    pub ty: TypeId,
    pub is_from_parent: bool,
    pub span: Span,
    pub id: Option<ParsedDeclId<ConstGenericDeclaration>>,
    pub expr: Option<ConstGenericExpr>,
}

impl HashWithEngines for ConstGenericParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let ConstGenericParameter {
            name, ty, id, expr, ..
        } = self;
        let type_engine = engines.te();
        type_engine.get(*ty).hash(state, engines);
        name.hash(state);
        if let Some(id) = id.as_ref() {
            let decl = engines.pe().get(id);
            decl.name.hash(state);
            decl.ty.hash(state);
        }
        match &expr {
            Some(expr) => {
                expr.hash(state);
            }
            None => {
                self.name.hash(state);
            }
        }
    }
}

impl EqWithEngines for ConstGenericParameter {}
impl PartialEqWithEngines for ConstGenericParameter {
    fn eq(&self, other: &Self, _ctx: &PartialEqWithEnginesContext) -> bool {
        self.name.as_str() == other.name.as_str()
    }
}

impl SubstTypes for ConstGenericParameter {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let mut has_changes = HasChanges::No;

        let Some(map) = ctx.type_subst_map else {
            return HasChanges::No;
        };

        // Check if it needs to be renamed
        if let Some(new_name) = map.const_generics_renaming.get(&self.name) {
            self.name = new_name.clone();
            has_changes = HasChanges::Yes;
        }

        // Check if it needs to be materialized
        if let Some(v) = map.const_generics_materialization.get(self.name.as_str()) {
            let handler = sway_error::handler::Handler::default();
            self.expr = Some(ConstGenericExpr::from_ty_expression(&handler, v).unwrap());
            has_changes = HasChanges::Yes;
        }

        has_changes
    }
}

impl IsConcrete for ConstGenericParameter {
    fn is_concrete(&self, _engines: &Engines) -> bool {
        todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
    }
}

impl PartialEq for ConstGenericParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name.as_str() == other.name.as_str()
    }
}

impl Eq for ConstGenericParameter {}

impl PartialOrd for ConstGenericParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.as_str().partial_cmp(other.name.as_str())
    }
}

impl Ord for ConstGenericParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.as_str().cmp(other.name.as_str())
    }
}
