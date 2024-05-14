use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*,
    error::module_can_be_changed,
    has_changes,
    language::{CallPath, Visibility},
    semantic_analysis::type_check_context::MonomorphizeHelper,
    transform,
    type_system::*,
    Namespace,
};

#[derive(Clone, Debug)]
pub struct TyStructDecl {
    pub call_path: CallPath,
    pub fields: Vec<TyStructField>,
    pub type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl Named for TyStructDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl EqWithEngines for TyStructDecl {}
impl PartialEqWithEngines for TyStructDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.call_path == other.call_path
            && self.fields.eq(&other.fields, ctx)
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyStructDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStructDecl {
            call_path,
            fields,
            type_parameters,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        call_path.hash(state);
        fields.hash(state, engines);
        type_parameters.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyStructDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.fields.subst(type_mapping, engines);
            self.type_parameters.subst(type_mapping, engines);
        }
    }
}

impl Spanned for TyStructDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyStructDecl {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }

    fn has_self_type_param(&self) -> bool {
        false
    }
}

impl TyStructDecl {
    /// Returns names of the [TyStructField]s of the struct `self` accessible in the given context.
    /// If `is_public_struct_access` is true, only the names of the public fields are returned, otherwise
    /// the names of all fields.
    /// Suitable for error reporting.
    pub(crate) fn accessible_fields_names(&self, is_public_struct_access: bool) -> Vec<Ident> {
        TyStructField::accessible_fields_names(&self.fields, is_public_struct_access)
    }

    /// Returns [TyStructField] with the given `field_name`, or `None` if the field with the
    /// name `field_name` does not exist.
    pub(crate) fn find_field(&self, field_name: &Ident) -> Option<&TyStructField> {
        self.fields.iter().find(|field| field.name == *field_name)
    }

    /// For the given `field_name` returns the zero-based index and the type of the field
    /// within the struct memory layout, or `None` if the field with the
    /// name `field_name` does not exist.
    pub(crate) fn get_field_index_and_type(&self, field_name: &Ident) -> Option<(u64, TypeId)> {
        // TODO-MEMLAY: Warning! This implementation assumes that fields are laid out in
        //              memory in the order of their declaration.
        //              This assumption can be changed in the future.
        self.fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == *field_name)
            .map(|(idx, field)| (idx as u64, field.type_argument.type_id))
    }

    /// Returns true if the struct `self` has at least one private field.
    pub(crate) fn has_private_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_private())
    }

    /// Returns true if the struct `self` has fields (it is not empty)
    /// and all fields are private.
    pub(crate) fn has_only_private_fields(&self) -> bool {
        !self.is_empty() && self.fields.iter().all(|field| field.is_private())
    }

    /// Returns true if the struct `self` does not have any fields.
    pub(crate) fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Provides information about the struct access within a particular [Namespace].
pub struct StructAccessInfo {
    /// True if the programmer who can change the code in the [Namespace]
    /// can also change the struct declaration.
    struct_can_be_changed: bool,
    /// True if the struct access is public, i.e., outside of the module in
    /// which the struct is defined.
    is_public_struct_access: bool,
}

impl StructAccessInfo {
    pub fn get_info(engines: &Engines, struct_decl: &TyStructDecl, namespace: &Namespace) -> Self {
        assert!(
            struct_decl.call_path.is_absolute,
            "The call path of the struct declaration must always be absolute."
        );

        let struct_can_be_changed =
            module_can_be_changed(engines, namespace, &struct_decl.call_path.prefixes);
        let is_public_struct_access =
            !namespace.module_is_submodule_of(engines, &struct_decl.call_path.prefixes, true);

        Self {
            struct_can_be_changed,
            is_public_struct_access,
        }
    }
}

impl From<StructAccessInfo> for (bool, bool) {
    /// Deconstructs `struct_access_info` into (`struct_can_be_changed`, `is_public_struct_access`)
    fn from(struct_access_info: StructAccessInfo) -> (bool, bool) {
        let StructAccessInfo {
            struct_can_be_changed,
            is_public_struct_access,
        } = struct_access_info;
        (struct_can_be_changed, is_public_struct_access)
    }
}

#[derive(Debug, Clone)]
pub struct TyStructField {
    pub visibility: Visibility,
    pub name: Ident,
    pub span: Span,
    pub type_argument: TypeArgument,
    pub attributes: transform::AttributesMap,
}

impl TyStructField {
    pub fn is_private(&self) -> bool {
        matches!(self.visibility, Visibility::Private)
    }

    pub fn is_public(&self) -> bool {
        matches!(self.visibility, Visibility::Public)
    }

    /// Returns [TyStructField]s from the `fields` that are accessible in the given context.
    /// If `is_public_struct_access` is true, only public fields are returned, otherwise
    /// all fields.
    pub(crate) fn accessible_fields(
        fields: &[TyStructField],
        is_public_struct_access: bool,
    ) -> impl Iterator<Item = &TyStructField> {
        fields
            .iter()
            .filter(move |field| !is_public_struct_access || field.is_public())
    }

    /// Returns names of the [TyStructField]s from the `fields` that are accessible in the given context.
    /// If `is_public_struct_access` is true, only the names of the public fields are returned, otherwise
    /// the names of all fields.
    /// Suitable for error reporting.
    pub(crate) fn accessible_fields_names(
        fields: &[TyStructField],
        is_public_struct_access: bool,
    ) -> Vec<Ident> {
        Self::accessible_fields(fields, is_public_struct_access)
            .map(|field| field.name.clone())
            .collect()
    }
}

impl Spanned for TyStructField {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl HashWithEngines for TyStructField {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStructField {
            visibility,
            name,
            type_argument,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        visibility.hash(state);
        name.hash(state);
        type_argument.hash(state, engines);
    }
}

impl EqWithEngines for TyStructField {}
impl PartialEqWithEngines for TyStructField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name && self.type_argument.eq(&other.type_argument, ctx)
    }
}

impl OrdWithEngines for TyStructField {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let TyStructField {
            name: ln,
            type_argument: lta,
            // these fields are not compared because they aren't relevant for ordering
            span: _,
            attributes: _,
            visibility: _,
        } = self;
        let TyStructField {
            name: rn,
            type_argument: rta,
            // these fields are not compared because they aren't relevant for ordering
            span: _,
            attributes: _,
            visibility: _,
        } = other;
        ln.cmp(rn).then_with(|| lta.cmp(rta, ctx))
    }
}

impl SubstTypes for TyStructField {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.type_argument.subst_inner(type_mapping, engines)
    }
}
