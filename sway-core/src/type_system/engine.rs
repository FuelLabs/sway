use crate::{
    concurrent_slab::{ConcurrentSlab, ListDisplay},
    decl_engine::*,
    engine_threading::*,
    type_system::priv_prelude::*,
};
use core::fmt::Write;
use hashbrown::{hash_map::RawEntryMut, HashMap};
use parking_lot::RwLock;
use std::sync::Arc;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    type_error::TypeError,
};
use sway_types::{integer_bits::IntegerBits, span::Span, ProgramId, SourceId};

use super::unify::unifier::UnifyKind;

#[derive(Debug, Default)]
pub struct TypeEngine {
    slab: ConcurrentSlab<TypeSourceInfo>,
    id_map: RwLock<HashMap<TypeSourceInfo, TypeId>>,
}

impl Clone for TypeEngine {
    fn clone(&self) -> Self {
        TypeEngine {
            slab: self.slab.clone(),
            id_map: RwLock::new(self.id_map.read().clone()),
        }
    }
}

impl TypeEngine {
    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert(
        &self,
        engines: &Engines,
        ty: TypeInfo,
        source_id: Option<&SourceId>,
    ) -> TypeId {
        let source_id = source_id.copied().or_else(|| info_to_source_id(&ty));
        let tsi = TypeSourceInfo {
            type_info: ty.clone().into(),
            source_id,
        };
        let mut id_map = self.id_map.write();

        let hash_builder = id_map.hasher().clone();
        let ty_hash = make_hasher(&hash_builder, engines)(&tsi);

        let raw_entry = id_map.raw_entry_mut().from_hash(ty_hash, |x| {
            x.eq(&tsi, &PartialEqWithEnginesContext::new(engines))
        });
        match raw_entry {
            RawEntryMut::Occupied(o) => return *o.get(),
            RawEntryMut::Vacant(_) if ty.can_change(engines.de()) => {
                TypeId::new(self.slab.insert(tsi))
            }
            RawEntryMut::Vacant(v) => {
                let type_id = TypeId::new(self.slab.insert(tsi.clone()));
                v.insert_with_hasher(ty_hash, tsi, type_id, make_hasher(&hash_builder, engines));
                type_id
            }
        }
    }

    /// Removes all data associated with `program_id` from the type engine.
    pub fn clear_program(&mut self, program_id: &ProgramId) {
        self.slab.retain(|_, tsi| match tsi.source_id {
            Some(source_id) => &source_id.program_id() != program_id,
            None => true,
        });
        self.id_map.write().retain(|tsi, _| match tsi.source_id {
            Some(source_id) => &source_id.program_id() != program_id,
            None => true,
        });
    }

    pub fn replace(&self, id: TypeId, new_value: TypeSourceInfo) {
        self.slab.replace(id.index(), new_value);
    }

    /// Performs a lookup of `id` into the [TypeEngine].
    pub fn get(&self, id: TypeId) -> Arc<TypeInfo> {
        self.slab.get(id.index()).type_info.clone()
    }

    /// Performs a lookup of `id` into the [TypeEngine] recursing when finding a
    /// [TypeInfo::Alias].
    pub fn get_unaliased(&self, id: TypeId) -> Arc<TypeInfo> {
        // A slight infinite loop concern if we somehow have self-referential aliases, but that
        // shouldn't be possible.
        let tsi = self.slab.get(id.index());
        match &*tsi.type_info {
            TypeInfo::Alias { ty, .. } => self.get_unaliased(ty.type_id),
            _ => tsi.type_info.clone(),
        }
    }

    /// Performs a lookup of `id` into the [TypeEngine] recursing when finding a
    /// [TypeInfo::Alias].
    pub fn get_unaliased_type_id(&self, id: TypeId) -> TypeId {
        // A slight infinite loop concern if we somehow have self-referential aliases, but that
        // shouldn't be possible.
        let tsi = self.slab.get(id.index());
        match &*tsi.type_info {
            TypeInfo::Alias { ty, .. } => self.get_unaliased_type_id(ty.type_id),
            _ => id,
        }
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify(
        &self,
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) {
        Self::unify_helper(
            handler,
            engines,
            received,
            expected,
            span,
            help_text,
            err_override,
            UnifyKind::Default,
        );
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a self type and `received`
    /// is not).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify_with_self(
        &self,
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) {
        Self::unify_helper(
            handler,
            engines,
            received,
            expected,
            span,
            help_text,
            err_override,
            UnifyKind::WithSelf,
        );
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify_with_generic(
        &self,
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) {
        Self::unify_helper(
            handler,
            engines,
            received,
            expected,
            span,
            help_text,
            err_override,
            UnifyKind::WithGeneric,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn unify_helper(
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
        unify_kind: UnifyKind,
    ) {
        if !UnifyCheck::coercion(engines).check(received, expected) {
            // create a "mismatched type" error unless the `err_override`
            // argument has been provided
            match err_override {
                Some(err_override) => {
                    handler.emit_err(err_override);
                }
                None => {
                    handler.emit_err(CompileError::TypeError(TypeError::MismatchedType {
                        expected: engines.help_out(expected).to_string(),
                        received: engines.help_out(received).to_string(),
                        help_text: help_text.to_string(),
                        span: span.clone(),
                        internal: format!(
                            "expected:[{:?}]; received:[{:?}]",
                            engines.help_out(expected),
                            engines.help_out(received),
                        ),
                    }));
                }
            }
            return;
        }

        let h = Handler::default();
        let unifier = Unifier::new(engines, help_text, unify_kind);
        unifier.unify(handler, received, expected, span);

        match err_override {
            Some(err_override) if h.has_errors() => {
                handler.emit_err(err_override);
            }
            _ => {
                handler.append(h);
            }
        }
    }

    pub(crate) fn to_typeinfo(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match &*self.get(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty.clone()),
        }
    }

    /// Return whether a given type still contains undecayed references to [TypeInfo::Numeric]
    pub(crate) fn contains_numeric(&self, decl_engine: &DeclEngine, type_id: TypeId) -> bool {
        match &&*self.get(type_id) {
            TypeInfo::Enum(decl_ref) => {
                decl_engine
                    .get_enum(decl_ref)
                    .variants
                    .iter()
                    .all(|variant_type| {
                        self.contains_numeric(decl_engine, variant_type.type_argument.type_id)
                    })
            }
            TypeInfo::Struct(decl_ref) => decl_engine
                .get_struct(decl_ref)
                .fields
                .iter()
                .any(|field| self.contains_numeric(decl_engine, field.type_argument.type_id)),
            TypeInfo::Tuple(fields) => fields
                .iter()
                .any(|field_type| self.contains_numeric(decl_engine, field_type.type_id)),
            TypeInfo::Array(elem_ty, _length) => {
                self.contains_numeric(decl_engine, elem_ty.type_id)
            }
            TypeInfo::Ptr(targ) => self.contains_numeric(decl_engine, targ.type_id),
            TypeInfo::Slice(targ) => self.contains_numeric(decl_engine, targ.type_id),
            TypeInfo::Ref {
                referenced_type, ..
            } => self.contains_numeric(decl_engine, referenced_type.type_id),
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Placeholder(..)
            | TypeInfo::TypeParam(..)
            | TypeInfo::StringArray(..)
            | TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(..)
            | TypeInfo::Storage { .. }
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Alias { .. }
            | TypeInfo::TraitType { .. } => false,
            TypeInfo::Numeric => true,
        }
    }

    /// Resolve all inner types that still are a [TypeInfo::Numeric] to a concrete `u64`
    pub(crate) fn decay_numeric(
        &self,
        handler: &Handler,
        engines: &Engines,
        type_id: TypeId,
        span: &Span,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = engines.de();

        match &&*self.get(type_id) {
            TypeInfo::Enum(decl_ref) => {
                for variant_type in &decl_engine.get_enum(decl_ref).variants {
                    self.decay_numeric(handler, engines, variant_type.type_argument.type_id, span)?;
                }
            }
            TypeInfo::Struct(decl_ref) => {
                for field in &decl_engine.get_struct(decl_ref).fields {
                    self.decay_numeric(handler, engines, field.type_argument.type_id, span)?;
                }
            }
            TypeInfo::Tuple(fields) => {
                for field_type in fields {
                    self.decay_numeric(handler, engines, field_type.type_id, span)?;
                }
            }
            TypeInfo::Array(elem_ty, _length) => {
                self.decay_numeric(handler, engines, elem_ty.type_id, span)?;
            }
            TypeInfo::Ptr(targ) => self.decay_numeric(handler, engines, targ.type_id, span)?,
            TypeInfo::Slice(targ) => self.decay_numeric(handler, engines, targ.type_id, span)?,
            TypeInfo::Ref {
                referenced_type, ..
            } => self.decay_numeric(handler, engines, referenced_type.type_id, span)?,
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Placeholder(..)
            | TypeInfo::TypeParam(..)
            | TypeInfo::StringSlice
            | TypeInfo::StringArray(..)
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(..)
            | TypeInfo::Storage { .. }
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Alias { .. }
            | TypeInfo::TraitType { .. } => {}
            TypeInfo::Numeric => {
                self.unify(
                    handler,
                    engines,
                    type_id,
                    self.insert(
                        engines,
                        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                        span.source_id(),
                    ),
                    span,
                    "",
                    None,
                );
            }
        }
        Ok(())
    }

    /// Pretty print method for printing the [TypeEngine]. This method is
    /// manually implemented to avoid implementation overhead regarding using
    /// [DisplayWithEngines].
    pub fn pretty_print(&self, _decl_engine: &DeclEngine, engines: &Engines) -> String {
        let mut builder = String::new();
        let mut list = vec![];
        for tsi in self.slab.values() {
            list.push(format!("{:?}", engines.help_out(&*tsi.type_info)));
        }
        let list = ListDisplay { list };
        write!(builder, "TypeEngine {{\n{list}\n}}").unwrap();
        builder
    }
}

/// Maps specific [TypeInfo] variants to a reserved [SourceId], returning `None` for non-mapped types.
fn info_to_source_id(ty: &TypeInfo) -> Option<SourceId> {
    match ty {
        TypeInfo::Unknown
        | TypeInfo::UnsignedInteger(_)
        | TypeInfo::Numeric
        | TypeInfo::Boolean
        | TypeInfo::B256
        | TypeInfo::RawUntypedPtr
        | TypeInfo::RawUntypedSlice
        | TypeInfo::StringSlice
        | TypeInfo::Contract
        | TypeInfo::StringArray(_)
        | TypeInfo::Array(_, _)
        | TypeInfo::Ref { .. } => Some(SourceId::reserved()),
        TypeInfo::Tuple(v) if v.is_empty() => Some(SourceId::reserved()),
        _ => None,
    }
}
