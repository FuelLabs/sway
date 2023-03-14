use core::fmt::Write;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use std::sync::RwLock;

use crate::concurrent_slab::ListDisplay;
use crate::{
    concurrent_slab::ConcurrentSlab, decl_engine::*, engine_threading::*, language::ty,
    namespace::Path, type_system::*, Namespace,
};

use sway_error::{error::CompileError, type_error::TypeError, warning::CompileWarning};
use sway_types::{span::Span, Ident, Spanned};

use super::unify::Unifier;
use super::unify_check::UnifyCheck;

#[derive(Debug, Default)]
pub struct TypeEngine {
    pub(super) slab: ConcurrentSlab<TypeInfo>,
    storage_only_types: ConcurrentSlab<TypeInfo>,
    id_map: RwLock<HashMap<TypeInfo, TypeId>>,
}

impl TypeEngine {
    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert(&self, decl_engine: &DeclEngine, ty: TypeInfo) -> TypeId {
        let mut id_map = self.id_map.write().unwrap();

        let engines = Engines::new(self, decl_engine);
        let hash_builder = id_map.hasher().clone();
        let ty_hash = make_hasher(&hash_builder, engines)(&ty);

        let raw_entry = id_map
            .raw_entry_mut()
            .from_hash(ty_hash, |x| x.eq(&ty, engines));
        match raw_entry {
            RawEntryMut::Occupied(o) => return *o.get(),
            RawEntryMut::Vacant(_) if ty.can_change(decl_engine) => {
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

    /// Denotes the given [TypeId] as being used with storage.
    pub(crate) fn set_type_as_storage_only(&self, id: TypeId) {
        self.storage_only_types.insert(self.get(id));
    }

    /// Checks if the given [TypeInfo] is a storage only type.
    pub(crate) fn is_type_info_storage_only(
        &self,
        decl_engine: &DeclEngine,
        ti: &TypeInfo,
    ) -> bool {
        self.storage_only_types
            .exists(|x| ti.is_subset_of(x, Engines::new(self, decl_engine)))
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    pub(crate) fn unify(
        &self,
        decl_engine: &DeclEngine,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        let engines = Engines::new(self, decl_engine);
        if !UnifyCheck::new(engines).check(received, expected) {
            // create a "mismatched type" error unless the `err_override`
            // argument has been provided
            let mut errors = vec![];
            match err_override {
                Some(err_override) => {
                    errors.push(err_override);
                }
                None => {
                    errors.push(CompileError::TypeError(TypeError::MismatchedType {
                        expected: engines.help_out(expected).to_string(),
                        received: engines.help_out(received).to_string(),
                        help_text: help_text.to_string(),
                        span: span.clone(),
                    }));
                }
            }
            return (vec![], errors);
        }
        let (warnings, errors) =
            normalize_err(Unifier::new(engines, help_text).unify(received, expected, span));
        if errors.is_empty() {
            (warnings, errors)
        } else if err_override.is_some() {
            // return the errors from unification unless the `err_override`
            // argument has been provided
            (warnings, vec![err_override.unwrap()])
        } else {
            (warnings, errors)
        }
    }

    /// Helper function for making the type of `expected` equivalent to
    /// `received` for instantiating algebraic data types.
    ///
    /// This method simply switches the arguments of `received` and `expected`
    /// and calls the `unify` method---the main purpose of this method is reduce
    /// developer overhead during implementation, as it is a little non-intuitive
    /// why `received` and `expected` should be switched.
    ///
    /// Let me explain, take this Sway code:
    ///
    /// ```ignore
    /// enum Option<T> {
    ///     Some(T),
    ///     None
    /// }
    ///
    /// struct Wrapper {
    ///     option: Option<bool>,
    /// }
    ///
    /// fn create_it<T>() -> Wrapper {
    ///     Wrapper {
    ///         option: Option::None
    ///     }
    /// }
    /// ```
    ///
    /// This is valid Sway code and we should expect it to compile. Here is the
    /// pseudo-code of roughly what we can expect from type inference:
    /// 1. `Option::None` is originally found to be of type `Option<T>` (because
    ///     it is not possible to know what `T` is just from the `None` case)
    /// 2. we call `unify_adt` with arguments `received` of type `Option<T>` and
    ///     `expected` of type `Option<bool>`
    /// 3. we switch `received` and `expected` and call the `unify` method
    /// 4. we perform type inference with a `received` type of `Option<bool>`
    ///     and an `expected` type of `Option<T>`
    /// 5. we perform type inference with a `received` type of `bool` and an
    ///     `expected` type of `T`
    /// 6. because we have called the `unify` method (and not the `unify_right`
    ///     method), we can replace `T` with `bool`
    ///
    /// What's important about this is flipping the arguments prioritizes
    /// unifying `expected`, meaning if both `received` and `expected` are
    /// generic types, then `expected` will be replaced with `received`.
    pub(crate) fn unify_adt(
        &self,
        decl_engine: &DeclEngine,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        let engines = Engines::new(self, decl_engine);
        if !UnifyCheck::new(engines).check(received, expected) {
            // create a "mismatched type" error unless the `err_override`
            // argument has been provided
            let mut errors = vec![];
            match err_override {
                Some(err_override) => {
                    errors.push(err_override);
                }
                None => {
                    errors.push(CompileError::TypeError(TypeError::MismatchedType {
                        expected: engines.help_out(expected).to_string(),
                        received: engines.help_out(received).to_string(),
                        help_text: help_text.to_string(),
                        span: span.clone(),
                    }));
                }
            }
            return (vec![], errors);
        }
        let (warnings, errors) = normalize_err(
            Unifier::new(engines, help_text)
                .flip_arguments()
                .unify(expected, received, span),
        );
        if errors.is_empty() {
            (warnings, errors)
        } else if err_override.is_some() {
            // return the errors from unification unless the `err_override`
            // argument has been provided
            (warnings, vec![err_override.unwrap()])
        } else {
            (warnings, errors)
        }
    }

    pub(crate) fn to_typeinfo(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match self.get(id) {
            TypeInfo::Unknown => {
                //panic!();
                Err(TypeError::UnknownType {
                    span: error_span.clone(),
                })
            }
            ty => Ok(ty),
        }
    }

    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
    /// enum, or a reference to a type parameter.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve(
        &self,
        decl_engine: &DeclEngine,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &mut Namespace,
        mod_path: &Path,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let engines = Engines::new(self, decl_engine);
        let module_path = type_info_prefix.unwrap_or(mod_path);
        let type_id = match self.get(type_id) {
            TypeInfo::Custom {
                call_path,
                type_arguments,
            } => {
                match namespace
                    .root()
                    .resolve_call_path_with_visibility_check(engines, module_path, &call_path)
                    .ok(&mut warnings, &mut errors)
                    .cloned()
                {
                    Some(ty::TyDeclaration::StructDeclaration {
                        name,
                        type_subst_list,
                        ..
                    }) => {
                        // Create a fresh list.
                        let mut subst_list = type_subst_list.fresh_copy();

                        // Monomorphize the list.
                        check!(
                            subst_list.monomorphize(
                                namespace,
                                engines,
                                mod_path,
                                &mut type_arguments.unwrap_or_default(),
                                enforce_type_arguments,
                                &name,
                                span
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // Create a new type id.
                        let type_id = engines.te().insert(decl_engine, TypeInfo::Struct(todo!()));

                        // Take any trait methods that apply to this type and
                        // copy them to the new type.
                        namespace.insert_trait_implementation_for_type(engines, type_id);

                        // return the id
                        type_id
                    }
                    Some(ty::TyDeclaration::EnumDeclaration {
                        name,
                        type_subst_list,
                        ..
                    }) => {
                        // Create a fresh list.
                        let mut subst_list = type_subst_list.fresh_copy();

                        // Monomorphize the list.
                        check!(
                            subst_list.monomorphize(
                                namespace,
                                engines,
                                mod_path,
                                &mut type_arguments.unwrap_or_default(),
                                enforce_type_arguments,
                                &name,
                                span
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // Create a new type id.
                        let type_id = engines.te().insert(decl_engine, TypeInfo::Struct(todo!()));

                        // Take any trait methods that apply to this type and
                        // copy them to the new type.
                        namespace.insert_trait_implementation_for_type(engines, type_id);

                        // return the id
                        type_id
                    }
                    Some(ty::TyDeclaration::GenericTypeForFunctionScope { type_id, .. }) => type_id,
                    _ => {
                        errors.push(CompileError::UnknownTypeName {
                            name: call_path.to_string(),
                            span: call_path.span(),
                        });
                        self.insert(decl_engine, TypeInfo::ErrorRecovery)
                    }
                }
            }
            TypeInfo::Array(mut elem_ty, n) => {
                elem_ty.type_id = check!(
                    self.resolve(
                        decl_engine,
                        elem_ty.type_id,
                        span,
                        enforce_type_arguments,
                        None,
                        namespace,
                        mod_path
                    ),
                    self.insert(decl_engine, TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                self.insert(decl_engine, TypeInfo::Array(elem_ty, n))
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        self.resolve(
                            decl_engine,
                            type_argument.type_id,
                            span,
                            enforce_type_arguments,
                            None,
                            namespace,
                            mod_path
                        ),
                        self.insert(decl_engine, TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                self.insert(decl_engine, TypeInfo::Tuple(type_arguments))
            }
            _ => type_id,
        };
        ok(type_id, warnings, errors)
    }

    /// Pretty print method for printing the [TypeEngine]. This method is
    /// manually implemented to avoid implementation overhead regarding using
    /// [DisplayWithEngines].
    pub fn pretty_print(&self, decl_engine: &DeclEngine) -> String {
        let engines = Engines::new(self, decl_engine);
        let mut builder = String::new();
        self.slab.with_slice(|elems| {
            let list = elems.iter().map(|type_info| engines.help_out(type_info));
            let list = ListDisplay { list };
            write!(builder, "TypeEngine {{\n{list}\n}}").unwrap();
        });
        builder
    }
}

fn normalize_err(
    (w, e): (Vec<CompileWarning>, Vec<TypeError>),
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    (w, e.into_iter().map(CompileError::from).collect())
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

impl EnforceTypeArguments {
    pub(crate) fn is_yes(&self) -> bool {
        matches!(self, EnforceTypeArguments::Yes)
    }

    pub(crate) fn is_no(&self) -> bool {
        !self.is_yes()
    }
}
