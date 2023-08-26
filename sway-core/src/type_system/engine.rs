use core::fmt::Write;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use std::sync::RwLock;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::integer_bits::IntegerBits;

use crate::concurrent_slab::ListDisplay;
use crate::{
    concurrent_slab::ConcurrentSlab, decl_engine::*, engine_threading::*, language::ty,
    namespace::Path, type_system::priv_prelude::*, Namespace,
};

use sway_error::{error::CompileError, type_error::TypeError};
use sway_types::{span::Span, Ident, Spanned};

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

    /// Given a `value` of type `T` that is able to be monomorphized and a set
    /// of `type_arguments`, monomorphize `value` with the `type_arguments`.
    ///
    /// When this function is called, it is passed a `T` that is a copy of some
    /// original declaration for `T` (let's denote the original with `[T]`).
    /// Because monomorphization happens at application time (e.g. function
    /// application), we want to be able to modify `value` such that type
    /// checking the application of `value` affects only `T` and not `[T]`.
    ///
    /// So, at a high level, this function does two things. It 1) performs the
    /// necessary work to refresh the relevant generic types in `T` so that they
    /// are distinct from the generics of the same name in `[T]`. And it 2)
    /// applies `type_arguments` (if any are provided) to the type parameters
    /// of `value`, unifying the types.
    ///
    /// There are 4 cases that are handled in this function:
    ///
    /// 1. `value` does not have type parameters + `type_arguments` is empty:
    ///     1a. return ok
    /// 2. `value` has type parameters + `type_arguments` is empty:
    ///     2a. if the [EnforceTypeArguments::Yes] variant is provided, then
    ///         error
    ///     2b. refresh the generic types with a [TypeSubstMapping]
    /// 3. `value` does have type parameters + `type_arguments` is nonempty:
    ///     3a. error
    /// 4. `value` has type parameters + `type_arguments` is nonempty:
    ///     4a. check to see that the type parameters and `type_arguments` have
    ///         the same length
    ///     4b. for each type argument in `type_arguments`, resolve the type
    ///     4c. refresh the generic types with a [TypeSubstMapping]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn monomorphize<T>(
        &self,
        handler: &Handler,
        engines: &Engines,
        value: &mut T,
        type_arguments: &mut [TypeArgument],
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
        namespace: &mut Namespace,
        mod_path: &Path,
    ) -> Result<(), ErrorEmitted>
    where
        T: MonomorphizeHelper + SubstTypes,
    {
        match (
            value.type_parameters().is_empty(),
            type_arguments.is_empty(),
        ) {
            (true, true) => Ok(()),
            (false, true) => {
                if let EnforceTypeArguments::Yes = enforce_type_arguments {
                    return Err(handler.emit_err(CompileError::NeedsTypeArguments {
                        name: value.name().clone(),
                        span: call_site_span.clone(),
                    }));
                }
                let type_mapping =
                    TypeSubstMap::from_type_parameters(engines, value.type_parameters());
                value.subst(&type_mapping, engines);
                Ok(())
            }
            (true, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| value.name().span());
                Err(handler.emit_err(CompileError::DoesNotTakeTypeArguments {
                    name: value.name().clone(),
                    span: type_arguments_span,
                }))
            }
            (false, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| value.name().span());
                if value.type_parameters().len() != type_arguments.len() {
                    return Err(
                        handler.emit_err(CompileError::IncorrectNumberOfTypeArguments {
                            given: type_arguments.len(),
                            expected: value.type_parameters().len(),
                            span: type_arguments_span,
                        }),
                    );
                }
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = self
                        .resolve(
                            handler,
                            engines,
                            type_argument.type_id,
                            &type_argument.span,
                            enforce_type_arguments,
                            None,
                            namespace,
                            mod_path,
                        )
                        .unwrap_or_else(|err| self.insert(engines, TypeInfo::ErrorRecovery(err)));
                }
                let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
                    value
                        .type_parameters()
                        .iter()
                        .map(|type_param| type_param.type_id)
                        .collect(),
                    type_arguments
                        .iter()
                        .map(|type_arg| type_arg.type_id)
                        .collect(),
                );
                // TODO treat Self: we should extend the type mapping with Self -> Placeholder
                value.subst(&type_mapping, engines);
                Ok(())
            }
        }
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
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
        Unifier::new(engines, help_text).unify(handler, received, expected, span);
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
            | TypeInfo::Str(..)
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
            | TypeInfo::Alias { .. } => false,
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
            | TypeInfo::Str(..)
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
            | TypeInfo::Alias { .. } => {}
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

    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
    /// enum, or a reference to a type parameter.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve(
        &self,
        handler: &Handler,
        engines: &Engines,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &mut Namespace,
        mod_path: &Path,
    ) -> Result<TypeId, ErrorEmitted> {
        let decl_engine = engines.de();
        let module_path = type_info_prefix.unwrap_or(mod_path);
        let type_id = match self.get(type_id) {
            TypeInfo::Custom {
                call_path,
                type_arguments,
            } => {
                match namespace
                    .root()
                    .resolve_call_path_with_visibility_check(
                        handler,
                        engines,
                        module_path,
                        &call_path,
                    )
                    .ok()
                    .cloned()
                {
                    Some(ty::TyDecl::StructDecl(ty::StructDecl {
                        decl_id: original_id,
                        ..
                    })) => {
                        // get the copy from the declaration engine
                        let mut new_copy = decl_engine.get_struct(&original_id);

                        // monomorphize the copy, in place
                        self.monomorphize(
                            handler,
                            engines,
                            &mut new_copy,
                            &mut type_arguments.unwrap_or_default(),
                            enforce_type_arguments,
                            span,
                            namespace,
                            mod_path,
                        )?;

                        // insert the new copy in the decl engine
                        let new_decl_ref = decl_engine.insert(new_copy);

                        // create the type id from the copy
                        let type_id = engines.te().insert(engines, TypeInfo::Struct(new_decl_ref));

                        // take any trait methods that apply to this type and copy them to the new type
                        namespace.insert_trait_implementation_for_type(engines, type_id);

                        // return the id
                        type_id
                    }
                    Some(ty::TyDecl::EnumDecl(ty::EnumDecl {
                        decl_id: original_id,
                        ..
                    })) => {
                        // get the copy from the declaration engine
                        let mut new_copy = decl_engine.get_enum(&original_id);

                        // monomorphize the copy, in place
                        self.monomorphize(
                            handler,
                            engines,
                            &mut new_copy,
                            &mut type_arguments.unwrap_or_default(),
                            enforce_type_arguments,
                            span,
                            namespace,
                            mod_path,
                        )?;

                        // insert the new copy in the decl engine
                        let new_decl_ref = decl_engine.insert(new_copy);

                        // create the type id from the copy
                        let type_id = engines.te().insert(engines, TypeInfo::Enum(new_decl_ref));

                        // take any trait methods that apply to this type and copy them to the new type
                        namespace.insert_trait_implementation_for_type(engines, type_id);

                        // return the id
                        type_id
                    }
                    Some(ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl {
                        decl_id: original_id,
                        ..
                    })) => {
                        let new_copy = decl_engine.get_type_alias(&original_id);

                        // TODO: monomorphize the copy, in place, when generic type aliases are
                        // supported

                        let type_id = new_copy.create_type_id(engines);
                        namespace.insert_trait_implementation_for_type(engines, type_id);

                        type_id
                    }
                    Some(ty::TyDecl::GenericTypeForFunctionScope(
                        ty::GenericTypeForFunctionScope { type_id, .. },
                    )) => type_id,
                    _ => {
                        let err = handler.emit_err(CompileError::UnknownTypeName {
                            name: call_path.to_string(),
                            span: call_path.span(),
                        });
                        self.insert(engines, TypeInfo::ErrorRecovery(err))
                    }
                }
            }
            TypeInfo::Array(mut elem_ty, n) => {
                elem_ty.type_id = self
                    .resolve(
                        handler,
                        engines,
                        elem_ty.type_id,
                        span,
                        enforce_type_arguments,
                        None,
                        namespace,
                        mod_path,
                    )
                    .unwrap_or_else(|err| self.insert(engines, TypeInfo::ErrorRecovery(err)));

                let type_id = self.insert(engines, TypeInfo::Array(elem_ty, n));

                // take any trait methods that apply to this type and copy them to the new type
                namespace.insert_trait_implementation_for_type(engines, type_id);

                type_id
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = self
                        .resolve(
                            handler,
                            engines,
                            type_argument.type_id,
                            span,
                            enforce_type_arguments,
                            None,
                            namespace,
                            mod_path,
                        )
                        .unwrap_or_else(|err| self.insert(engines, TypeInfo::ErrorRecovery(err)));
                }

                let type_id = self.insert(engines, TypeInfo::Tuple(type_arguments));

                // take any trait methods that apply to this type and copy them to the new type
                namespace.insert_trait_implementation_for_type(engines, type_id);

                type_id
            }
            _ => type_id,
        };
        Ok(type_id)
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

pub(crate) trait MonomorphizeHelper {
    fn name(&self) -> &Ident;
    fn type_parameters(&self) -> &[TypeParameter];
}

/// This type is used to denote if, during monomorphization, the compiler
/// should enforce that type arguments be provided. An example of that
/// might be this:
///
/// ```ignore
/// struct Point<T> {
///   x: u64,
///   y: u64
/// }
///
/// fn add<T>(p1: Point<T>, p2: Point<T>) -> Point<T> {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
///
/// `EnforeTypeArguments` would require that the type annotations
/// for `p1` and `p2` contain `<...>`. This is to avoid ambiguous definitions:
///
/// ```ignore
/// fn add(p1: Point, p2: Point) -> Point {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
#[derive(Clone, Copy)]
pub(crate) enum EnforceTypeArguments {
    Yes,
    No,
}
