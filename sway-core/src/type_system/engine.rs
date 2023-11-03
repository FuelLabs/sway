use core::fmt::Write;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use std::sync::RwLock;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::integer_bits::IntegerBits;

use crate::concurrent_slab::ListDisplay;
use crate::{
    concurrent_slab::ConcurrentSlab, decl_engine::*, engine_threading::*,
    type_system::priv_prelude::*,
};

use sway_error::{error::CompileError, type_error::TypeError};
use sway_types::span::Span;

use super::unify::unifier::UnifyKind;

#[derive(Debug, Default)]
pub struct TypeEngine {
    pub(super) slab: ConcurrentSlab<TypeInfo>,
    id_map: RwLock<HashMap<TypeInfo, TypeId>>,
}

impl TypeEngine {
    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert(&self, engines: &Engines, ty: TypeInfo) -> TypeId {
        let mut id_map = self.id_map.write().unwrap();

        let hash_builder = id_map.hasher().clone();
        let ty_hash = make_hasher(&hash_builder, engines)(&ty);

        let raw_entry = id_map
            .raw_entry_mut()
            .from_hash(ty_hash, |x| x.eq(&ty, engines));
        match raw_entry {
            RawEntryMut::Occupied(o) => return *o.get(),
            RawEntryMut::Vacant(_) if ty.can_change(engines.de()) => {
                TypeId::new(self.slab.insert(ty))
            }
            RawEntryMut::Vacant(v) => {
                let type_id = TypeId::new(self.slab.insert(ty.clone()));
                v.insert_with_hasher(ty_hash, ty, type_id, make_hasher(&hash_builder, engines));
                type_id
            }
        }
    }

    pub fn replace(&self, id: TypeId, engines: &Engines, new_value: TypeInfo) {
        let prev_value = self.slab.get(id.index());
        self.slab.replace(id, &prev_value, new_value, engines);
    }

    /// Performs a lookup of `id` into the [TypeEngine].
    pub fn get(&self, id: TypeId) -> TypeInfo {
        self.slab.get(id.index())
    }

    /// Performs a lookup of `id` into the [TypeEngine] recursing when finding a
    /// [TypeInfo::Alias].
    pub fn get_unaliased(&self, id: TypeId) -> TypeInfo {
        // A slight infinite loop concern if we somehow have self-referential aliases, but that
        // shouldn't be possible.
        match self.slab.get(id.index()) {
            TypeInfo::Alias { ty, .. } => self.get_unaliased(ty.type_id),
            ty_info => ty_info,
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
        self.unify_helper(
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
        self.unify_helper(
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
        self.unify_helper(
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
        &self,
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
        match self.get(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty),
        }
    }

    /// Return whether a given type still contains undecayed references to [TypeInfo::Numeric]
    pub(crate) fn contains_numeric(&self, decl_engine: &DeclEngine, type_id: TypeId) -> bool {
        match &self.get(type_id) {
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
            TypeInfo::Unknown
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

        match &self.get(type_id) {
            TypeInfo::Enum(decl_ref) => {
                for variant_type in decl_engine.get_enum(decl_ref).variants.iter() {
                    self.decay_numeric(handler, engines, variant_type.type_argument.type_id, span)?;
                }
            }
            TypeInfo::Struct(decl_ref) => {
                for field in decl_engine.get_struct(decl_ref).fields.iter() {
                    self.decay_numeric(handler, engines, field.type_argument.type_id, span)?;
                }
            }
            TypeInfo::Tuple(fields) => {
                for field_type in fields {
                    self.decay_numeric(handler, engines, field_type.type_id, span)?;
                }
            }
            TypeInfo::Array(elem_ty, _length) => {
                self.decay_numeric(handler, engines, elem_ty.type_id, span)?
            }
            TypeInfo::Ptr(targ) => self.decay_numeric(handler, engines, targ.type_id, span)?,
            TypeInfo::Slice(targ) => self.decay_numeric(handler, engines, targ.type_id, span)?,

            TypeInfo::Unknown
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
                    self.insert(engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
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
        self.slab.with_slice(|elems| {
            let list = elems
                .iter()
                .map(|type_info| format!("{:?}", engines.help_out(type_info)));
            let list = ListDisplay { list };
            write!(builder, "TypeEngine {{\n{list}\n}}").unwrap();
        });
        builder
    }
}
